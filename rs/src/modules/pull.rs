use std::process::Command;
use std::str::FromStr;
use clap::{Args, Error};
use crate::config::MWUtilConfig;
use crate::types::RepoType;

#[derive(Clone, Debug, PartialEq)]
pub enum PullRepoType {
    Core,
    Config,
    Enum(RepoType),
}

#[derive(Args)]
pub struct PullArgs {
    /// The type of the repo
    #[clap(value_parser = parse_repo_type)]
    pub repo_type: PullRepoType,
    /// The name of the local repo
    pub name: Option<String>,
}

pub fn execute(config: &MWUtilConfig, args: PullArgs) -> anyhow::Result<()> {
    let dir = match args.repo_type {
        PullRepoType::Core => config.core_dir.clone(),
        PullRepoType::Config => config.config_dir.clone(),
        PullRepoType::Enum(other) => config.base_dir
            .join(other.get_plural_name())
            .join(args.name.expect("You must specify a name for this type of repository."))
    };

    Command::new("git")
        .arg("pull")
        .current_dir(dir)
        .status()?;

    Ok(())
}


fn parse_repo_type(s: &str) -> Result<PullRepoType, Error> {
    match s.to_lowercase().as_str() {
        "core" => Ok(PullRepoType::Core),
        "config" => Ok(PullRepoType::Config),
        other => Ok(PullRepoType::Enum(other.parse().expect("Invalid RepoType"))),
    }
}