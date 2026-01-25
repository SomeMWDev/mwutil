use crate::config::MWUtilConfig;
use anyhow::bail;
use clap::{Args, Subcommand};
use regex::Regex;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use std::{env, fs};

#[derive(Args)]
pub struct SecurityArgs {
    #[command(subcommand)]
    command: SecurityCommand,
}

#[derive(Subcommand)]
pub enum SecurityCommand {
    /// Creates a patch based on the last commit
    CreatePatch(CreatePatchArgs),
}

#[derive(Args)]
pub struct CreatePatchArgs {
    /// The name of the patch to create
    name: Option<String>,

    /// Uses the branch name as the patch name
    #[arg(long)]
    use_branch_name: bool,
}

pub fn execute(config: Option<&MWUtilConfig>, args: SecurityArgs) -> anyhow::Result<()> {
    match args.command {
        SecurityCommand::CreatePatch(create_patch_args) => create_patch(config, create_patch_args),
    }
}

fn create_patch(config: Option<&MWUtilConfig>, args: CreatePatchArgs) -> anyhow::Result<()> {
    let folder: PathBuf = match config {
        Some(c) => {
            if let Some(folder_str) = &c.security_patch_folder {
                PathBuf::from_str(folder_str)?
            } else {
                env::current_dir()?
            }
        },
        None => env::current_dir()?,
    };
    if !folder.exists() {
        fs::create_dir_all(&folder)?;
    }
    let name: String;
    if let Some(n) = args.name {
        name = n;
    } else {
        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Failed to determine current branch. Error: {stderr}");
        }
        let branch_name = String::from_utf8(output.stdout)?.trim().to_string();
        if branch_name.is_empty() {
            bail!("Current brancch name is empty (detached HEAD?). Please provide --name.");
        }
        println!("No patch name provided. Current branch: {branch_name}");
        if args.use_branch_name {
            name = branch_name;
            println!("Using branch name since --use-branch-name was specified.");
        } else if Regex::new(r"^T[0-9]{4,10}$")?.is_match(branch_name.as_str()) {
            name = branch_name;
            println!("Using branch name since a task ID was detected in it.");
        } else {
            bail!("Please specify a patch name or use the --use-branch-name option!");
        }
    }

    let safe_name = name.replace(['/', '\\'], "-");
    let patch_file = folder.join(format!("{safe_name}.patch"));
    let status = Command::new("git")
        .args(["format-patch", "HEAD^", "--output", patch_file.to_string_lossy().as_ref()])
        .status()?;
    if !status.success() {
        bail!("Failed to create patch!");
    }

    println!("Patch created at {}", patch_file.to_string_lossy().as_ref());

    Ok(())
}
