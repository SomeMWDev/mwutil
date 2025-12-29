use crate::config::{load_mwutil_config, DBType, MWUtilConfig};
use crate::constants::MEDIAWIKI_CONTAINER;
use crate::exec::{create_db_command, run_sql_query, DbCommandDatabase, DbCommandType, DbCommandUser};
use crate::modules::recreate;
use crate::modules::recreate::RecreateArgs;
use anyhow::{anyhow, Context};
use clap::{Args, Subcommand};
use clap_complete::ArgValueCompleter;
use clap_complete::CompletionCandidate;
use console::style;
use indicatif::ProgressBar;
use regex::Regex;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

const ALLOWED_DUMP_REGEX: &str = r"^[A-Za-z0-9\-._]+$";

#[derive(Args)]
pub struct DbArgs {
    #[command(subcommand)]
    command: DbCommand,
}

#[derive(Subcommand)]
pub enum DbCommand {
    /// Allows managing database dumps
    Dump(DumpArgs),
    /// Switches to a different database type
    Switch(SwitchArgs),
}

#[derive(Args)]
pub struct DumpArgs {
    #[command(subcommand)]
    sub_command: DumpSubCommand,
}

#[derive(Args)]
pub struct DumpSubArgs {
    /// The name of the dump
    #[arg(add = ArgValueCompleter::new(dump_completer))]
    name: String,
}

#[derive(Subcommand)]
pub enum DumpSubCommand {
    /// Creates a database dump
    Create(DumpSubArgs),
    /// Deletes a database dump
    Delete(DumpSubArgs),
    /// Imports a database dump
    Import(DumpSubArgs),
    /// Lists all database dumps
    List,
}

#[derive(Args)]
pub struct SwitchArgs {
    /// The type to switch to
    #[clap(value_parser)]
    to: DBType,
}

pub fn execute(config: &MWUtilConfig, args: DbArgs) -> anyhow::Result<()> {
    match args.command {
        DbCommand::Dump(dump_args) => execute_dump_command(config, dump_args),
        DbCommand::Switch(switch_args) => switch(config, switch_args),
    }
}

pub fn execute_dump_command(config: &MWUtilConfig, args: DumpArgs)-> anyhow::Result<()> {
    match args.sub_command {
        DumpSubCommand::Create(create_args) => create_dump(config, create_args),
        DumpSubCommand::Delete(delete_args) => delete_dump(config, delete_args),
        DumpSubCommand::Import(import_args) => import_dump(config, import_args),
        DumpSubCommand::List => list_dumps(config),
    }
}

pub fn create_dump(config: &MWUtilConfig, args: DumpSubArgs) -> anyhow::Result<()> {
    let dump_file = get_dump(config, &args.name, Existence::MustNotExist)?;
    let steps = 2;

    let spinner = create_spinner("Dumping database...", 1, steps);
    let out = create_db_command(config, DbCommandType::Dump, DbCommandUser::Mw, None, None, None)?
        .output()
        .context("Failed to dump database!")?;
    spinner.finish();

    let spinner = create_spinner("Writing dump to file...", 2, steps);
    fs::write(&dump_file, out.stdout)
        .context("Failed to write dump to file!")?;
    spinner.finish();

    println!(
        "{} dump at {}!",
        style("Created").green(),
        dump_file.to_str().unwrap_or("[unknown]"),
    );

    Ok(())
}

pub fn delete_dump(config: &MWUtilConfig, args: DumpSubArgs) -> anyhow::Result<()> {
    let dump_file = get_dump(config, &args.name, Existence::MustExist)?;

    fs::remove_file(&dump_file)?;
    println!(
        "{} dump at {}!",
        style("Deleted").green(),
        dump_file.to_str().unwrap_or("[unknown]"),
    );

    Ok(())
}

pub fn import_dump(config: &MWUtilConfig, args: DumpSubArgs) -> anyhow::Result<()> {
    let dump_file = get_dump(config, &args.name, Existence::MustExist)?;
    let bytes = fs::read(dump_file).context("Failed to read dump file")?;

    let steps = 4;
    let spinner = create_spinner("Dropping database", 1, steps);
    run_sql_query(
        config,
        DbCommandUser::Mw,
        Some(DbCommandDatabase::None),
        format!(
            "DROP DATABASE `{}`;",
            config.mw_database.clone().ok_or_else(|| anyhow!("MW database not set!"))?
        ).as_str(),
    ).context("Failed to drop database")?;
    spinner.finish();

    let spinner = create_spinner("Creating database", 2, steps);
    run_sql_query(
        config,
        DbCommandUser::Mw,
        Some(DbCommandDatabase::None),
        format!(
            "CREATE DATABASE `{}`;",
            config.mw_database.clone().ok_or_else(|| anyhow!("MW database not set!"))?
        ).as_str()
    ).context("Failed to create database")?;
    spinner.finish();

    let spinner = create_spinner("Importing dump", 3, steps);
    let mut process = create_db_command(
        config,
        DbCommandType::Query,
        DbCommandUser::Mw,
        None,
        Some(&["-T".into()]),
        None
    )?
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to spawn DB process")?;
    process.stdin.as_mut().ok_or(anyhow!("Failed to copy process stdin!"))?.write_all(&bytes)?;
    process.wait()?;
    spinner.finish();

    let spinner = create_spinner("Restarting MW container", 4, steps);
    recreate::execute(config, RecreateArgs {
        container: Some(String::from(MEDIAWIKI_CONTAINER))
    })?;
    spinner.finish();

    Ok(())
}

pub fn list_dumps(config: &MWUtilConfig) -> anyhow::Result<()> {
    let dump_files = get_all_dump_files(config)
        .ok_or(anyhow!("Failed to get all dump files!"))?;
    for file in dump_files {
        println!("{}", file.file_stem().unwrap().to_str().unwrap());
    }

    Ok(())
}

pub fn switch(config: &MWUtilConfig, args: SwitchArgs) -> anyhow::Result<()> {
    if config.db_type == args.to {
        println!("Already using {}!", style(args.to).red());
        return Ok(());
    }
    let steps = 4;
    let spinner = create_spinner("Creating dump...", 1, steps);

    spinner.finish();


    Ok(())
}

fn create_spinner(
    text: &str,
    step: u8,
    max: u8,
) -> ProgressBar {
    println!("{} {text}...", style(format!("[{step}/{max}]")).bold().dim());

    let spinner = ProgressBar::new_spinner();
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner
}

fn get_all_dump_files(config: &MWUtilConfig) -> Option<impl Iterator<Item = PathBuf>> {
    let Ok(files) = fs::read_dir(&config.dump_dir) else {
        return None;
    };
    Some(
        files.flatten()
            .filter_map(|file| {
                let path = file.path();
                if path.is_file() && path.extension() == Some("sql".as_ref()) {
                    Some(path)
                } else {
                    None
                }
            })
    )
}

fn dump_completer(_current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    let Ok(config) = load_mwutil_config(false) else {
        return vec![];
    };
    let Some(files) = get_all_dump_files(&config) else {
        return vec![];
    };
    files
        .map(|p| p.file_stem().map(|s| CompletionCandidate::new(s)))
        .flatten()
        .collect()
}

#[derive(PartialEq)]
enum Existence {
    Ignore,
    MustExist,
    MustNotExist,
}

fn get_dump(
    config: &MWUtilConfig,
    name: &String,
    existence_check: Existence
) -> anyhow::Result<PathBuf> {
    let re = Regex::new(ALLOWED_DUMP_REGEX)?;
    if !re.is_match(name) {
        anyhow::bail!(
            "{} dump name \"{}\"!",
            style("Invalid").red(),
            name
        );
    }

    if !config.dump_dir.exists() {
        fs::create_dir(config.dump_dir.as_path())?;
        println!("{} dump directory.", style("Created").green());
    }
    let dump_file = config.dump_dir.join(format!("{}.sql", name));
    if existence_check == Existence::MustExist && !dump_file.exists() {
        anyhow::bail!(
            "Dump file {} at {}!",
            style("does not exist").red(),
            dump_file.to_str().unwrap_or_default()
        );
    } else if existence_check == Existence::MustNotExist && dump_file.exists() {
        anyhow::bail!(
            "Dump file {} at {}!",
            style("already exists").red(),
            dump_file.to_str().unwrap_or_default()
        );
    }
    Ok(dump_file)
}
