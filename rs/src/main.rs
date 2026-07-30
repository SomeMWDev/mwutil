use crate::config::{load_mwutil_config, MWUtilConfig};
use crate::modules::bash::BashArgs;
use crate::modules::clone::CloneArgs;
use crate::modules::composer::ComposerArgs;
use crate::modules::container_action::ContainerActionArgs;
use crate::modules::db::DbArgs;
use crate::modules::lint::LintArgs;
use crate::modules::npm::NpmArgs;
use crate::modules::opensearch::OpenSearchArgs;
use crate::modules::pull::PullArgs;
use crate::modules::reset::ResetArgs;
use crate::modules::run::{RunArgs, RunShorthandArgs};
use crate::modules::security::SecurityArgs;
use crate::modules::setup_repo::SetupRepoArgs;
use crate::modules::sql::SqlArgs;
use crate::types::RepoOrigin;
use anyhow::bail;
use clap::{CommandFactory, Parser, Subcommand};
use crate::modules::farm::FarmArgs;
use crate::modules::watch::WatchArgs;

mod config;
mod modules;
mod utils;
mod exec;
mod types;
mod constants;
mod farm;

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
    /// Starts a bash shell in a container
    Bash(BashArgs),
    /// Clones a repository from GitHub or Gerrit
    Clone(CloneArgs),
    /// Runs composer update
    Composer(ComposerArgs),
    /// Allows managing the database
    Db(DbArgs),
    /// Stops containers
    Down(ContainerActionArgs),
    /// Allows managing a wiki farm
    Farm(FarmArgs),
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
    Recreate(ContainerActionArgs),
    /// Resets various parts of the local dev environment
    Reset(ResetArgs),
    /// Runs a maintenance script
    Run(RunArgs),
    /// Allows creating and pushing security patches
    Security(SecurityArgs),
    /// Sets up a local repository that was cloned from GitHub
    SetupGithub(SetupRepoArgs),
    /// Sets up git-review in a local repository that was cloned from gerrit
    SetupGerrit(SetupRepoArgs),
    /// Starts an interactive PHP shell
    Shell(RunShorthandArgs),
    /// Starts an interactive SQL shell
    Sql(SqlArgs),
    /// Starts containers
    Up(ContainerActionArgs),
    /// Runs update.php
    Update(RunShorthandArgs),
    /// Watches a file and copies it to the clipboard if it changes
    Watch(WatchArgs),
}

fn main() -> anyhow::Result<()> {
    clap_complete::CompleteEnv::with_factory(Cli::command).complete();

    let cli = Cli::parse();
    let config = load_mwutil_config(cli.debug);

    cli.module.run_globally(config.as_ref())
}

impl Modules {
    pub fn run(self, config: &MWUtilConfig) -> anyhow::Result<()> {
        self.run_globally(Ok(config))
    }

    pub fn run_globally(self, config: Result<&MWUtilConfig, &anyhow::Error>) -> anyhow::Result<()> {
        if let Err(e) = config && !self.works_globally() {
            bail!("The selected module only works inside of an mw-dev-kit environment.\n{e}");
        }
        match self {
            Modules::Bash(args) => modules::bash::execute(config.unwrap(), args),
            Modules::Clone(args) => modules::clone::execute(config.unwrap(), args),
            Modules::Composer(args) => modules::composer::execute(config.unwrap(), args),
            Modules::Db(args) => modules::db::execute(config.unwrap(), args),
            Modules::Down(args) => modules::container_action::down(config.unwrap(), args),
            Modules::Farm(args) => modules::farm::execute(config.unwrap(), args),
            Modules::Info => modules::info::execute(config.unwrap()),
            Modules::Lint(args) => modules::lint::execute(config.unwrap(), args, true),
            Modules::ListRepoRemotes => modules::list_repo_remotes::execute(config.unwrap()),
            Modules::Npm(args) => modules::npm::execute(args),
            Modules::OpenSearch(args) => modules::opensearch::execute(config.unwrap(), args),
            Modules::Pull(args) => modules::pull::execute(config.unwrap(), args),
            Modules::Recreate(args)=> modules::container_action::recreate(config.unwrap(), args),
            Modules::Reset(args) => modules::reset::execute(config.unwrap(), args),
            Modules::Run(args) => modules::run::execute(config.unwrap(), args),
            Modules::Security(args) => modules::security::execute(config.ok(), args),
            Modules::SetupGerrit(args) => modules::setup_repo::execute(config.unwrap(), args, RepoOrigin::Gerrit),
            Modules::SetupGithub(args) => modules::setup_repo::execute(config.unwrap(), args, RepoOrigin::Github),
            Modules::Shell(args) => modules::run::execute(config.unwrap(), RunArgs {
                script: "shell".into(),
                extra_args: args.extra_args,
                farm_command_args: args.farm_command_args
            }),
            Modules::Sql(args) => modules::sql::execute(config.unwrap(), args),
            Modules::Update(args) => {
                let mut extra_args = vec!["--quick".into()];
                extra_args.append(args.extra_args.clone().as_mut());
                modules::run::execute(config.unwrap(), RunArgs {
                    script: "update".into(),
                    extra_args,
                    farm_command_args: args.farm_command_args
                })
            },
            Modules::Up(args) => modules::container_action::up(config.unwrap(), args),
            Modules::Watch(args) => modules::watch::execute(args),
        }
    }

    fn works_globally(&self) -> bool {
        matches!(self, Modules::Npm(_) | Modules::Security(_) | Modules::Watch(_))
    }
}
