use anyhow::bail;
use crate::config::{load_mwutil_config, MWUtilConfig};
use crate::modules::bash::BashArgs;
use crate::modules::clone::CloneArgs;
use crate::modules::composer::ComposerArgs;
use crate::modules::db::DbArgs;
use crate::modules::pull::PullArgs;
use crate::modules::recreate::RecreateArgs;
use crate::modules::run::RunArgs;
use crate::modules::sql::SqlArgs;
use clap::{CommandFactory, Parser, Subcommand};
use crate::modules::down::DownArgs;
use crate::modules::lint::LintArgs;
use crate::modules::npm::NpmArgs;
use crate::modules::opensearch::OpenSearchArgs;
use crate::modules::security::SecurityArgs;

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
    /// Allows managing the database
    Db(DbArgs),
    /// Stops containers
    Down(DownArgs),
    /// Prints info about the environment
    Info,
    /// Runs a linter
    Lint(LintArgs),
    /// Lists the remotes of all local repos in the environment
    ListRepoRemotes,
    /// Runs npm install
    Npm(NpmArgs),
    /// Allows managing the OpenSearch instance
    OpenSearch(OpenSearchArgs),
    /// Pulls a local repository
    Pull(PullArgs),
    /// Recreates containers
    Recreate(RecreateArgs),
    /// Runs a maintenance script
    Run(RunArgs),
    /// Allows creating and pushing security patches
    Security(SecurityArgs),
    /// Sets up a local repository that was cloned from GitHub
    SetupGithub,
    /// Sets up git-review in a local repository that was cloned from gerrit
    SetupGerrit,
    /// Starts an interactive PHP shell
    Shell,
    /// Starts an interactive SQL shell
    Sql(SqlArgs),
    /// Starts all containers
    Up,
    /// Runs update.php
    Update,
}

fn main() -> anyhow::Result<()> {
    clap_complete::CompleteEnv::with_factory(Cli::command).complete();

    let cli = Cli::parse();
    let config = load_mwutil_config(cli.debug);

    run_module(cli.module, config.as_ref().ok())
}

pub fn run_module(module: Modules, config: Option<&MWUtilConfig>) -> anyhow::Result<()> {
    if config.is_none() && !module.works_globally() {
        bail!("The selected module only works inside of an mw-dev-kit environment!");
    }
    match module {
        Modules::AddGerritSSHKey => modules::add_gerrit_ssh_key::execute(config.unwrap()),
        Modules::Bash(args) => modules::bash::execute(config.unwrap(), args),
        Modules::Clone(args) => modules::clone::execute(config.unwrap(), args),
        Modules::Composer(args) => modules::composer::execute(config.unwrap(), args),
        Modules::Db(args) => modules::db::execute(config.unwrap(), args),
        Modules::Down(args) => modules::down::execute(config.unwrap(), args),
        Modules::Info => modules::info::execute(config.unwrap()),
        Modules::Lint(args) => modules::lint::execute(config.unwrap(), args, true),
        Modules::ListRepoRemotes => modules::list_repo_remotes::execute(config.unwrap()),
        Modules::Npm(args) => modules::npm::execute(args),
        Modules::OpenSearch(args) => modules::opensearch::execute(config.unwrap(), args),
        Modules::Pull(args) => modules::pull::execute(config.unwrap(), args),
        Modules::Recreate(args) => modules::recreate::execute(config.unwrap(), args),
        Modules::Run(args) => modules::run::execute(config.unwrap(), args),
        Modules::Security(args) => modules::security::execute(config, args),
        Modules::SetupGerrit => modules::setup_gerrit::execute(config.unwrap(), None),
        Modules::SetupGithub => modules::setup_github::execute(config.unwrap(), None),
        Modules::Shell => modules::run::execute(config.unwrap(), RunArgs {
            script: "shell".into(),
            ..Default::default()
        }),
        Modules::Sql(args) => modules::sql::execute(config.unwrap(), args),
        Modules::Update => modules::run::execute(config.unwrap(), RunArgs {
            script: "update".into(),
            extra_args: vec!["--quick".into()],
        }),
        Modules::Up => modules::up::execute(config.unwrap()),
    }
}

impl Modules {
    fn works_globally(&self) -> bool {
        match self {
            Modules::Security(_) => true,
            _ => false,
        }
    }
}
