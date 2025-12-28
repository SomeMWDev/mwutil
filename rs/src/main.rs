use clap::{CommandFactory, Parser, Subcommand};
use console::Term;
use crate::config::load_mwutil_config;
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
enum Modules {
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
}

fn main() -> anyhow::Result<()> {
    clap_complete::CompleteEnv::with_factory(Cli::command).complete();

    let cli = Cli::parse();

    let config = load_mwutil_config(cli.debug);

    let term = Term::stdout();
    // TODO don't unwrap config
    match cli.module {
        Modules::AddGerritSSHKey => modules::add_gerrit_ssh_key::execute(config.unwrap())?,
        Modules::Bash(args) => modules::bash::execute(args)?,
        Modules::Clone(args) => modules::clone::execute(config.unwrap(), args)?,
        Modules::Composer(args) => modules::composer::execute(args)?,
        Modules::Run(args) => modules::run::execute(config.unwrap(), args)?,
    }

    Ok(())
}
