use crate::config::{find_base_dir, MWUtilConfig};
use crate::exec::ContainerSupport;
use crate::types::Container;
use crate::utils::get_core_version;
use clap::Args;
use clap_complete::ArgValueCompleter;
use clap_complete::CompletionCandidate;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Args, Default)]
pub struct RunArgs {
    /// The name of the maintenance script to run
    #[arg(add = ArgValueCompleter::new(script_completer))]
    pub script: String,

    /// Additional arguments to pass to the script
    #[arg(trailing_var_arg = true)]
    pub extra_args: Vec<String>,
}

pub fn execute(config: &MWUtilConfig, args: RunArgs) -> anyhow::Result<()> {
    let core_version = get_core_version(config);
    let mut cmd: Command;
    if core_version.map(|v|v.minor).unwrap_or(0) >= 40 {
        cmd = Command::new("maintenance/run");
        cmd.arg(args.script);
    } else {
        cmd = Command::new("php");
        if args.script.contains(".php") {
            cmd.arg(args.script);
        } else {
            cmd.arg("maintenance/".to_owned() + args.script.as_str() + ".php");
        }
    }
    cmd.args(args.extra_args);

    cmd.in_container(config, Container::MediaWiki, None)?
        .status()
        .ok();
    Ok(())
}

fn script_completer(_current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    let Some(base_dir) = find_base_dir() else {
        return vec![];
    };
    let mut result = vec![];

    add_scripts_from_directory(
        base_dir.join("core").join("maintenance"),
        None,
        &mut result
    );
    if let Ok(extensions) = fs::read_dir(base_dir.join("extensions")) {
        for extension in extensions.flatten() {
            add_scripts_from_directory(
                extension.path().join("maintenance"),
                Some(extension.file_name().to_string_lossy().as_ref()),
                &mut result
            );
        }
    }

    result
}

fn add_scripts_from_directory(folder: PathBuf, prefix: Option<&str>, result: &mut Vec<CompletionCandidate>) {
    let scripts = fs::read_dir(folder);
    if let Ok(entries) = scripts {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension() == Some("php".as_ref()) && let Some(name) = path.file_name() {
                if let Some(prefix) = prefix {
                    let name = name.to_str().expect("Failed to decode file name string");
                    result.push(CompletionCandidate::new(format!("{prefix}:{name}")));
                } else {
                    result.push(CompletionCandidate::new(name));
                }
            }
        }
    }
}
