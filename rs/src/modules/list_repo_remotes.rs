use crate::config::MWUtilConfig;
use crate::types::RepoType;
use crate::utils::capitalize;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use strum::IntoEnumIterator;

pub fn execute(config: &MWUtilConfig) -> anyhow::Result<()> {
    for repo_type in RepoType::iter() {
        println!("{}:", capitalize(repo_type.get_plural_name().as_str()));
        list_folder(config.base_dir.join(repo_type.get_plural_name()))?;
    }
    Ok(())
}

fn list_folder(folder: PathBuf) -> anyhow::Result<()> {
    let mut remotes: HashMap<String, String> = HashMap::new();
    if let Ok(entries) = fs::read_dir(folder) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            if !path.join(".git").exists() {
                continue;
            }
            let status_res = Command::new("git")
                .args(["config", "--get", "remote.origin.url"])
                .current_dir(&path)
                .output();
            let Ok(status) = status_res else {
                continue;
            };
            let mut remote = String::from_utf8(status.stdout)?;
            remote.pop();
            remotes.insert(
                path.file_stem()
                    .map(OsStr::to_string_lossy)
                    .map(|c| c.to_string())
                    .unwrap_or("[Invalid]".to_string()),
                remote
            );
        }
    }
    let mut sorted: Vec<_> = remotes.iter().collect();
    sorted.sort_by_key(|&(key, _)| key);

    for (key, value) in sorted {
        println!("{key}: {value}");
    }
    println!();
    Ok(())
}
