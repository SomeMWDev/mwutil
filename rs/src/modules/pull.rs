use crate::config::MWUtilConfig;
use crate::types::RepoType;
use clap::{Args, ValueEnum};
use std::process::Command;

// todo un-hardcode RepoType values here
#[derive(Clone, Debug, PartialEq, ValueEnum)]
pub enum PullRepoType {
    Core,
    Config,
    Extension,
    Service,
    Skin,
    Tool,
}

#[derive(Args)]
pub struct PullArgs {
    /// The type of the repo
    pub repo_type: PullRepoType,
    /// The name of the local repo
    pub name: Option<String>,
}

pub fn execute(config: &MWUtilConfig, args: PullArgs) -> anyhow::Result<()> {
    let mut dir = match args.repo_type {
        PullRepoType::Core => config.core_dir.clone(),
        PullRepoType::Config => config.config_dir.clone(),
        PullRepoType::Extension => config.base_dir.join(RepoType::Extension.get_plural_name()),
        PullRepoType::Skin => config.base_dir.join(RepoType::Skin.get_plural_name()),
        PullRepoType::Service => config.base_dir.join(RepoType::Service.get_plural_name()),
        PullRepoType::Tool => config.base_dir.join(RepoType::Tool.get_plural_name()),
    };
    if let Some(name) = args.name {
        dir = dir.join(name);
    }

    Command::new("git")
        .arg("pull")
        .current_dir(dir)
        .status()?;

    Ok(())
}

impl PullRepoType {
    pub fn from_repo_type(repo_type: &RepoType) -> Self {
        match repo_type {
            RepoType::Extension => PullRepoType::Extension,
            RepoType::Skin => PullRepoType::Skin,
            RepoType::Service => PullRepoType::Service,
            RepoType::Tool => PullRepoType::Tool,
        }
    }
}
