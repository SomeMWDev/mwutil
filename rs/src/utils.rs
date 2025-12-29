use crate::config::{load_mwutil_config, MWUtilConfig};
use crate::exec::create_docker_compose_command;
use crate::types::MWVersion;
use anyhow::anyhow;
use clap_complete::CompletionCandidate;
use regex::Regex;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn container_completer(_current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    let config = load_mwutil_config(false)
        .expect("Failed to load config!");
    create_docker_compose_command(&config)
        .args(["ps", "--services"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|s| {
            s.lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(CompletionCandidate::new)
                .collect()
        })
        .unwrap_or_default()
}

pub fn get_core_version(config: &MWUtilConfig) -> Option<MWVersion> {
    let version_file = config.core_dir.join("includes").join("Defines.php");
    let contents = fs::read_to_string(version_file).ok()?;
    let re = Regex::new(r"'MW_VERSION', '([a-zA-Z0-9\-.]+)'")
        .unwrap();

    re.captures(contents.as_str())
        .map(|c|c.get(0).unwrap().as_str())
        .and_then(MWVersion::parse)
}

pub fn set_git_config(option: &str, value: &str, repo_folder: &PathBuf) -> anyhow::Result<()> {
    Command::new("git")
        .args(["config", "--local", option, value])
        .current_dir(repo_folder)
        .status()
        .map_err(|e| anyhow!("Failed to set git option: {}", e))?;
    Ok(())
}

