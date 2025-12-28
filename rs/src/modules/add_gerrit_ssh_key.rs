use crate::config::MWUtilConfig;
use anyhow::{anyhow, Context};
use console::style;
use std::process::Command;

pub fn execute(config: MWUtilConfig) -> anyhow::Result<()> {
    let key = config.gerrit_ssh_key
        .ok_or_else(|| anyhow!("GERRIT_SSH_KEY is not set in configuration!"))?;

    if config.debug {
        println!("SSH Key: {}", key);
    }

    let status = Command::new("ssh-add")
        .arg(key)
        .status()
        .context("failed to execute process")?;

    if status.success() {
        println!("{} added SSH key", style("Successfully").green());
    } else {
        println!("ssh-add {} with non-zero status", style("exited").red());
    }

    Ok(())
}
