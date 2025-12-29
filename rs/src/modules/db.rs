use clap_complete::ArgValueCompleter;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use anyhow::{Context};
use clap::{Args, Subcommand};
use clap_complete::CompletionCandidate;
use console::style;
use indicatif::ProgressBar;
use regex::Regex;
use crate::config::{load_mwutil_config, DBType, MWUtilConfig};
use crate::exec::{create_db_command, DbCommandType, DbCommandUser};

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
        _ => Ok(()) // TODO implement
    }
}

pub fn create_dump(config: &MWUtilConfig, args: DumpSubArgs) -> anyhow::Result<()> {
    let dump_file = get_dump(config, &args.name, Existence::MustNotExist)?;
    let steps = 2;

    let spinner = create_spinner("Dumping database...", 1, steps);
    let out = create_db_command(config, DbCommandType::Dump, DbCommandUser::Default)?
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

fn dump_completer(_current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    let Ok(config) = load_mwutil_config(false) else {
        return vec![];
    };
    let Ok(files) = fs::read_dir(config.dump_dir) else {
        return vec![];
    };
    files.flatten()
        .filter_map(|file| {
            let path = file.path();
            if path.is_file() && path.extension() == Some("sql".as_ref()) && let Some(name) = path.file_stem() {
                Some(CompletionCandidate::new(name))
            } else {
                None
            }
        })
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
