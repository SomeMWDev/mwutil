use crate::config::MWUtilConfig;
use crate::utils::set_git_config;
use anyhow::anyhow;
use std::env;
use std::path::PathBuf;
use std::process::Command;

pub fn execute(config: &MWUtilConfig, repo_folder: Option<PathBuf>) -> anyhow::Result<()> {
    let repo_folder = repo_folder.unwrap_or(env::current_dir()?);
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
    set_git_config(
        "gitreview.username",
        config.git_username.clone().ok_or_else(|| anyhow!("git-review username not set!"))?.as_str(),
        &repo_folder
    )?;
    set_git_config("gitreview.remote", "origin", &repo_folder)?;
    Command::new("git")
        .args(["review", "-s", "--verbose"])
        .current_dir(repo_folder)
        .status()?;
    Ok(())
}
