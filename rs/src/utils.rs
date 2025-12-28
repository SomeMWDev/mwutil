use std::fs;
use std::process::Command;
use clap_complete::CompletionCandidate;
use regex::Regex;
use crate::config::MWUtilConfig;
use crate::types::MWVersion;

pub fn container_completer(_current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    Command::new("docker")
        .args(["compose", "--env-file", "config/.env", "ps", "--services"])
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
