use clap::{CommandFactory, Parser, Subcommand};
use console::Term;
use crate::config::{load_mwutil_config, MWUtilConfig};
use crate::modules::bash::BashArgs;

mod config;
mod modules;
mod utils;
mod exec;

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
    }

    Ok(())
}
