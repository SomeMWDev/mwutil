use crate::config::{find_base_dir, update_env_var, update_profiles, DBType, MWUtilConfig};
use crate::constants::{ALLOWED_DUMP_REGEX, MEDIAWIKI_CONTAINER};
use crate::exec::{create_db_command, run_sql_query, DbCommandDatabase, DbCommandType, DbCommandUser};
use crate::modules::container_action::ContainerActionArgs;
use crate::utils::SpinnerSequence;
use crate::Modules;
use anyhow::{anyhow, bail, Context};
use clap::{Args, Subcommand};
use clap_complete::ArgValueCompleter;
use clap_complete::CompletionCandidate;
use console::style;
use dialoguer::Confirm;
use rand::distr::Alphanumeric;
use rand::Rng;
use regex::Regex;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::thread::sleep;
use std::time::Duration;

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
    /// Deletes all database dumps
    DeleteAll,
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
        DumpSubCommand::DeleteAll => delete_all_dumps(config),
        DumpSubCommand::Import(import_args) => import_dump(config, import_args),
        DumpSubCommand::List => list_dumps(config),
    }
}

pub fn create_dump(config: &MWUtilConfig, args: DumpSubArgs) -> anyhow::Result<()> {
    let dump_file = get_dump(config, &args.name, Existence::MustNotExist)?;

    let mut spinner = SpinnerSequence::new(1, "Creating dump");
    let mut cmd = create_db_command(
        config,
        DbCommandType::Dump,
        DbCommandUser::Mw,
        Some(&["--skip-set-charset", "--default-character-set=utf8mb4"]),
        None,
        None
    )?;
    let file = File::create(&dump_file)
        .context("Failed to create dump file!")?;
    let status = cmd
        .stdout(Stdio::from(file))
        .status()
        .context("Failed to dump database!")?;

    if !status.success() {
        bail!("Failed to dump database: command returned non-zero status {:?}", status.code())
    }
    spinner.finish();

    println!(
        "{} dump at {}!",
        style("Created").green(),
        dump_file.to_string_lossy(),
    );

    Ok(())
}

pub fn delete_dump(config: &MWUtilConfig, args: DumpSubArgs) -> anyhow::Result<()> {
    let dump_file = get_dump(config, &args.name, Existence::MustExist)?;

    fs::remove_file(&dump_file)?;
    println!(
        "{} dump at {}!",
        style("Deleted").green(),
        dump_file.to_string_lossy(),
    );

    Ok(())
}

pub fn delete_all_dumps(config: &MWUtilConfig) -> anyhow::Result<()> {
    let dump_files: Vec<PathBuf> = get_all_dump_files(&config.dump_dir)
        .ok_or_else(|| anyhow!("Failed to get all dump files!"))?
        .collect();

    let confirmation = Confirm::new()
        .with_prompt(format!("Do you want to continue and delete {} dump files?", dump_files.len()))
        .interact()?;

    if !confirmation {
        return Ok(())
    }

    for file in dump_files {
        fs::remove_file(&file)?;
        println!("Deleted {}", file.to_string_lossy());
    }

    Ok(())
}

pub fn drop_mw_database(config: &MWUtilConfig) -> anyhow::Result<()> {
    let status = run_sql_query(
        config,
        DbCommandUser::Mw,
        Some(DbCommandDatabase::None),
        format!(
            "DROP DATABASE IF EXISTS `{}`;",
            config.mw_database.clone().ok_or_else(|| anyhow!("MW database not set!"))?
        ).as_str(),
    ).context("Failed to drop database")?;
    if !status.success() {
        bail!("Failed to drop database: Command returned an error!")
    }
    Ok(())
}

pub fn import_dump(config: &MWUtilConfig, args: DumpSubArgs) -> anyhow::Result<()> {
    let dump_file = get_dump(config, &args.name, Existence::MustExist)?;
    let bytes = fs::read(dump_file).context("Failed to read dump file")?;

    let mut spinner = SpinnerSequence::new(4, "Dropping database");
    drop_mw_database(config)?;

    spinner.next("Creating database");
    run_sql_query(
        config,
        DbCommandUser::Root,
        Some(DbCommandDatabase::None),
        format!(
            "CREATE DATABASE `{}`;",
            config.mw_database.clone().ok_or_else(|| anyhow!("MW database not set!"))?
        ).as_str()
    ).context("Failed to create database")?;

    spinner.next("Importing dump");
    let mut process = create_db_command(
        config,
        DbCommandType::Query,
        DbCommandUser::Root,
        None,
        Some(&["-T".into()]),
        None
    )?
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to spawn DB process")?;
    process.stdin.as_mut().ok_or_else(|| anyhow!("Failed to copy process stdin!"))?.write_all(&bytes)?;
    process.wait()?;

    spinner.next("Restarting MW container");
    Modules::Recreate(ContainerActionArgs {
        container: Some(String::from(MEDIAWIKI_CONTAINER))
    }).run(config)?;
    spinner.finish();

    Ok(())
}

pub fn list_dumps(config: &MWUtilConfig) -> anyhow::Result<()> {
    let dump_files = get_all_dump_files(&config.dump_dir)
        .ok_or_else(|| anyhow!("Failed to get all dump files!"))?;
    for file in dump_files {
        println!("{}", file.file_stem()
            .map(OsStr::to_string_lossy)
            .map(|c| c.to_string())
            .unwrap_or("[Invalid]".to_string())
        );
    }

    Ok(())
}

pub fn switch(config: &MWUtilConfig, args: SwitchArgs) -> anyhow::Result<()> {
    if config.db_type == args.to {
        println!("Already using {}!", style(args.to).red());
        return Ok(());
    }

    let mut spinner = SpinnerSequence::new(7, "Creating dump");

    let dump_name: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();
    create_dump(config, DumpSubArgs { name: dump_name.clone() })?;

    spinner.next("Updating environment variables");
    let mut profiles = config.compose_profiles.clone();
    profiles.retain(|p| p != config.db_type.get_container_name());
    profiles.push(args.to.get_container_name().into());
    let mut new_config = config.clone();
    update_profiles(&mut new_config, &profiles)?;
    new_config.db_type = args.to.clone();
    update_env_var(config, "MWC_DB_TYPE", args.to.get_container_name())?;
    update_env_var(config, "MWC_DB_HOST", args.to.get_container_name())?;

    spinner.next("Stopping old container");
    Modules::Down(ContainerActionArgs {
        container: Some(config.db_type.get_container_name().into())
    }).run(config)?;

    spinner.next("Starting new container");
    Modules::Up(ContainerActionArgs {
        container: Some(args.to.get_container_name().into())
    }).run(config)?;

    spinner.next("Waiting for the database to be ready");
    loop {
        let res = run_sql_query(
            &new_config,
            DbCommandUser::Mw,
            Some(DbCommandDatabase::Mw),
            "SELECT 1;"
        );
        if res.is_ok_and(|s|s.success()) {
            println!("Database is ready!");
            break
        }
        sleep(Duration::from_secs(1))
    }

    spinner.next("Importing dump");
    import_dump(&new_config, DumpSubArgs {
        name: dump_name.clone(),
    })?;

    spinner.next("Deleting dump");
    delete_dump(&new_config, DumpSubArgs {
        name: dump_name.clone(),
    })?;

    spinner.finish();

    Ok(())
}

fn get_all_dump_files(dump_dir: &Path) -> Option<impl Iterator<Item = PathBuf>> {
    let Ok(files) = fs::read_dir(dump_dir) else {
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
    let Some(base_dir) = find_base_dir() else {
        return vec![];
    };
    let dump_dir = base_dir.join("dumps");
    let Some(files) = get_all_dump_files(&dump_dir) else {
        return vec![];
    };
    files
        .filter_map(|p| p.file_stem().map(CompletionCandidate::new))
        .collect()
}

#[derive(PartialEq)]
enum Existence {
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
        bail!(
            "{} dump name \"{}\"!",
            style("Invalid").red(),
            name
        );
    }

    if !config.dump_dir.exists() {
        fs::create_dir_all(config.dump_dir.as_path())?;
        println!("{} dump directory.", style("Created").green());
    }
    let dump_file = config.dump_dir.join(format!("{}.sql", name));
    if existence_check == Existence::MustExist && !dump_file.exists() {
        bail!(
            "Dump file {} at {}!",
            style("does not exist").red(),
            dump_file.to_string_lossy()
        );
    } else if existence_check == Existence::MustNotExist && dump_file.exists() {
        bail!(
            "Dump file {} at {}!",
            style("already exists").red(),
            dump_file.to_string_lossy()
        );
    }
    Ok(dump_file)
}
