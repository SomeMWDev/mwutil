use clap::{arg, CommandFactory, Parser, Subcommand};
use console::Term;
use crate::config::{load_mwutil_config, MWUtilConfig};
use crate::modules::bash::BashArgs;
use crate::modules::clone::CloneArgs;
use crate::modules::composer::ComposerArgs;
use crate::modules::run::RunArgs;

mod config;
mod modules;
mod utils;
mod exec;
mod types;
mod constants;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[arg(short, long)]
    debug: bool,

    #[command(subcommand)]
    module: Modules,
}

#[derive(Subcommand)]
pub enum Modules {
    /// Adds the Gerrit SSH key to the SSH agent
    AddGerritSSHKey,
    /// Starts a bash shell in a container
    Bash(BashArgs),
    /// Clones a repository from GitHub or Gerrit
    Clone(CloneArgs),
    /// Runs composer update
    Composer(ComposerArgs),
    /// Runs a maintenance script
    Run(RunArgs),
    /// Runs update.php
    Update,
}

fn main() -> anyhow::Result<()> {
    clap_complete::CompleteEnv::with_factory(Cli::command).complete();

    let cli = Cli::parse();
    let config = load_mwutil_config(cli.debug);

    run_module(cli.module, config.as_ref().ok())?;

    Ok(())
}

pub fn run_module(module: Modules, config: Option<&MWUtilConfig>) -> anyhow::Result<()> {
    // TODO don't unwrap config
    match module {
        Modules::AddGerritSSHKey => modules::add_gerrit_ssh_key::execute(config.unwrap())?,
        Modules::Bash(args) => modules::bash::execute(args)?,
        Modules::Clone(args) => modules::clone::execute(config.unwrap(), args)?,
        Modules::Composer(args) => modules::composer::execute(args)?,
        Modules::Run(args) => modules::run::execute(config.unwrap(), args)?,
        Modules::Update => modules::run::execute(config.unwrap(), RunArgs {
            script: "update".into(),
            extra_args: vec!["--quick".into()],
        })?,
    }

    Ok(())
}
