use anyhow::{anyhow, bail, Context};
use clap::{Args, Subcommand};
use crate::config::MWUtilConfig;
use crate::exec::{run_sql_query, DbCommandDatabase, DbCommandUser};
use crate::modules::{db, reset};

#[derive(Args)]
pub struct FarmArgs {
    #[command(subcommand)]
    command: FarmCommand,
}

#[derive(Subcommand)]
pub enum FarmCommand {
    /// Install a new wiki
    Install(InstallArgs),
}

#[derive(Args)]
pub struct InstallArgs {
    /// The DB name of the wiki that should be installed
    db_name: String,
}

pub fn execute(config: &MWUtilConfig, farm_args: FarmArgs) -> anyhow::Result<()> {
    match farm_args.command {
        FarmCommand::Install(args) => install_wiki(config, args),
    }
}

fn install_wiki(config: &MWUtilConfig, args: InstallArgs) -> anyhow::Result<()> {
    if !args.db_name.ends_with("wiki") {
        // TODO maybe we want to allow other suffixes?
        bail!("The DB name must end with 'wiki'!");
    }

    let status = run_sql_query(
        config,
        DbCommandUser::Root,
        Some(DbCommandDatabase::None),
        format!(
            "CREATE DATABASE IF NOT EXISTS `{}`;",
            args.db_name,
        ).as_str()
    ).context("Failed to create database")?;
    if !status.success() {
        bail!("Failed to create database! Exit code: {:?}", status.code());
    }

    db::grant_privileges(config, &args.db_name)?;

    println!("Created database.");

    reset::reset_database(config, &args.db_name, true)?;

    Ok(())
}
