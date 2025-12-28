use std::env;
use std::path::PathBuf;
use std::process::Command;
use anyhow::anyhow;
use crate::config::MWUtilConfig;

pub fn execute(config: &MWUtilConfig, repo_folder: Option<PathBuf>) -> anyhow::Result<()> {
    // TODO fix unwrap
    let repo_folder = repo_folder.unwrap_or(env::current_dir()?);
    set_git_config("user.email", config.git_email.clone().unwrap().as_str(), &repo_folder)?;
    set_git_config("user.name", config.git_username.clone().unwrap().as_str(), &repo_folder)?;
    set_git_config(
        format!("url.\"ssh://{}@gerrit.wikimedia.org:29418/\".insteadOf", config.gerrit_username.clone().unwrap()).as_str(),
        "\"https://gerrit.wikimedia.org/r/\"",
        &repo_folder
    )?;
    set_git_config("gitreview.username", config.git_username.clone().unwrap().as_str(), &repo_folder)?;
    set_git_config("gitreview.remote", "origin", &repo_folder)?;
    Command::new("git")
        .args(["review", "-s", "--verbose"])
        .current_dir(repo_folder)
        .status()?;
    Ok(())
}

fn set_git_config(option: &str, value: &str, repo_folder: &PathBuf) -> anyhow::Result<()> {
    Command::new("git")
        .args(["config", "--local", option, value])
        .current_dir(repo_folder)
        .status()
        .map_err(|e| anyhow!("Failed to set git option: {}", e))?;
    Ok(())
}
