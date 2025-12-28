use clap::{CommandFactory, Parser, Subcommand};
use console::Term;
use crate::config::{load_mwutil_config, MWUtilConfig};

mod config;
mod modules;

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
    AddGerritSSHKey
}

fn main() -> anyhow::Result<()> {
    clap_complete::CompleteEnv::with_factory(Cli::command).complete();

    let cli = Cli::parse();

    let config = load_mwutil_config(cli.debug).unwrap();

    let term = Term::stdout();
    match cli.module {
        Modules::AddGerritSSHKey => modules::add_gerrit_ssh_key::execute(config)?,
    }

    Ok(())
}
