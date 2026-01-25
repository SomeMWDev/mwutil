use crate::config::MWUtilConfig;
use crate::types::RepoOrigin;
use crate::utils::set_git_config;
use anyhow::{anyhow, bail};
use clap::Args;
use std::env;
use std::path::PathBuf;
use std::process::Command;

#[derive(Args, Default)]
pub struct SetupRepoArgs {
    /// The folder of the repo
    pub folder: Option<PathBuf>,
}

pub fn execute(config: &MWUtilConfig, args: SetupRepoArgs, repo_origin: RepoOrigin) -> anyhow::Result<()> {
    match repo_origin {
        RepoOrigin::Gerrit => setup_gerrit(config, args),
        RepoOrigin::Github => setup_github(config, args),
    }
}

pub fn setup_gerrit(config: &MWUtilConfig, args: SetupRepoArgs) -> anyhow::Result<()> {
    let repo_folder = args.folder.unwrap_or(env::current_dir()?);
    set_git_config(
        "user.email",
        config.git_email.clone().ok_or_else(|| anyhow!("Git email not set!"))?.as_str(),
        &repo_folder
    )?;
    set_git_config(
        "user.name",
        config.git_username.clone().ok_or_else(|| anyhow!("Git name not set!"))?.as_str(),
        &repo_folder
    )?;
    set_git_config(
        &format!(
            "url.\"ssh://{}@gerrit.wikimedia.org:29418/\".insteadOf",
            config.gerrit_username.clone().ok_or_else(|| anyhow!("Gerrit username not set!"))?.as_str()
        ),
        "\"https://gerrit.wikimedia.org/r/\"",
        &repo_folder
    )?;
    // ToDo should we really use git_username here?
    set_git_config(
        "gitreview.username",
        config.git_username.clone().ok_or_else(|| anyhow!("git-review username not set!"))?.as_str(),
        &repo_folder
    )?;
    set_git_config("gitreview.remote", "origin", &repo_folder)?;
    let status = Command::new("git")
        .args(["review", "-s", "--verbose"])
        .current_dir(repo_folder)
        .status()?;
    if !status.success() {
        bail!("git review setup failed!");
    }
    Ok(())
}

pub fn setup_github(config: &MWUtilConfig, args: SetupRepoArgs) -> anyhow::Result<()> {
    let repo_folder = args.folder.unwrap_or(env::current_dir()?);
    set_git_config(
        "user.email",
        config.git_email.clone().ok_or_else(|| anyhow!("Git email not set!"))?.as_str(),
        &repo_folder
    )?;
    set_git_config(
        "user.name",
        config.git_username.clone().ok_or_else(|| anyhow!("Git name not set!"))?.as_str(),
        &repo_folder
    )?;
    println!("Done!");
    Ok(())
}
