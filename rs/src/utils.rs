use crate::config::{load_mwutil_config, MWUtilConfig};
use crate::exec::create_docker_compose_command;
use crate::types::MWVersion;
use anyhow::Context;
use clap_complete::CompletionCandidate;
use console::style;
use indicatif::ProgressBar;
use regex::Regex;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

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
        .context("Failed to set git option!")?;
    Ok(())
}

pub struct SpinnerSequence {
    pub cur: u8,
    pub max: u8,
    pub last: Option<ProgressBar>,
}

impl SpinnerSequence {
    pub fn next(&mut self, text: &str) {
        if let Some(spinner) = &self.last {
            spinner.finish();
        }
        self.cur += 1;

        println!("{} {text}...", style(format!("[{}/{}]", self.cur, self.max)).bold().dim());

        let spinner = ProgressBar::new_spinner();
        spinner.enable_steady_tick(Duration::from_millis(100));
        self.last = Some(spinner);
    }

    pub fn finish(self) {
        if let Some(spinner) = self.last {
            spinner.finish();
        }
    }

    pub fn new(max: u8, initial_text: &str) -> Self {
        let mut seq = Self {
            cur: 0,
            max,
            last: None,
        };
        seq.next(initial_text);
        seq
    }
}

pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
