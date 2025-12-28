use std::process::Command;
use clap_complete::CompletionCandidate;

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
