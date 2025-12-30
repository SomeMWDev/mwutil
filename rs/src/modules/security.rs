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
    let folder = config
        .map(|c| {
            c.security_patch_folder
                .clone()
                .as_deref()
                .map(|s| PathBuf::from_str(s).unwrap())
        })
        .unwrap_or_default()
        .unwrap_or(env::current_dir()?);
    if !folder.exists() {
        fs::create_dir(&folder)?;
    }
    let name: String;
    if args.name.is_some() {
        name = args.name.unwrap();
    } else {
        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .output()?;
        let branch_name = String::from_utf8(output.stdout)?.trim().to_string();
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

    let patch_file = folder.join(format!("{name}.patch"));
    let status = Command::new("git")
        .args(["format-patch", "HEAD^", "--output", patch_file.to_str().unwrap()])
        .status()?;
    if !status.success() {
        bail!("Failed to create patch!");
    }

    println!("Patch created at {}", patch_file.to_str().unwrap());

    Ok(())
}
