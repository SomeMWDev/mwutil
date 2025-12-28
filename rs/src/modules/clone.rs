use std::os::unix::prelude::ExitStatusExt;
use std::path::Path;
use std::process::{Command, ExitStatus};
use crate::config::MWUtilConfig;
use clap::{Args};
use regex::Regex;
use crate::exec::CommandExt;
use crate::modules::{composer, pull, setup_gerrit, setup_github};
use crate::modules::composer::ComposerArgs;
use crate::{run_module, Modules};
use crate::modules::pull::{PullArgs, PullRepoType};
use crate::types::{CloneMethod, RepoOrigin, RepoType};

#[derive(Args)]
pub struct CloneArgs {
    /// The type of the repo
    #[clap(value_parser)]
    repo_type: RepoType,

    /// The origin of the repo
    #[clap(value_parser)]
    repo_origin: RepoOrigin,

    /// The repo to clone
    repo: String,

    /// The name for the local repo
    name: Option<String>,

    /// Pull with --depth=1
    #[clap(long, alias = "quick")]
    shallow: bool,

    /// The method to use for cloning
    #[clap(short, long, value_parser, default_value_t = CloneMethod::Ssh)]
    method: CloneMethod,

    /// Run composer update after cloning
    #[clap(long)]
    composer: bool,

    /// The branch to clone
    #[clap(short, long)]
    branch: Option<String>,
}

pub fn execute(config: &MWUtilConfig, args: CloneArgs) -> anyhow::Result<()> {
    let repo_data = get_repo_data(config, &args.repo_type, args.repo_origin.clone(), args.repo, args.method);
    let name = args.name.unwrap_or(repo_data.0);
    let url = repo_data.1;
    let target_folder = config.base_dir.join(args.repo_type.get_plural_name());
    let (status, _stdout, stderr) = clone(&url, &name, &target_folder, args.shallow, args.branch.as_ref());
    println!("Git clone exited with status: {}", status);
    if status.into_raw() == 32768 {
        if stderr.contains("already exists and is not an empty directory") {
            println!("Directory already exists and is not empty. Pulling instead...");
            return pull::execute(config, PullArgs {
                repo_type: PullRepoType::Enum(args.repo_type),
                name: Some(name),
            });
        }
        println!("Attempting to clone default branch instead...");
        let (status, stdout, stderr) = clone(&url, &name, &target_folder, args.shallow, None);
        println!("Git clone exited with status: {}", status);
        if !status.success() {
            return Err(anyhow::anyhow!("Git clone failed: {}\n{}", stdout, stderr));
        }
    }

    let repo_folder = target_folder.join(name);

    if args.repo_origin == RepoOrigin::Gerrit {
        setup_gerrit::execute(config, Some(repo_folder))?;
    } else if args.repo_origin == RepoOrigin::Github {
        setup_github::execute(config, Some(repo_folder))?;
    }

    if args.composer {
        composer::execute(config, ComposerArgs::default())?;
    }
    if args.repo_type == RepoType::Extension {
        run_module(Modules::Update, Some(config))?;
    }

    Ok(())
}

fn get_repo_data(
    config: &MWUtilConfig,
    repo_type: &RepoType,
    repo_origin: RepoOrigin,
    repo: String,
    clone_method: CloneMethod,
) -> (String, String) {
    match repo_origin {
        RepoOrigin::Gerrit => {
            let name: String;
            let repo_identifier: String;

            if repo.contains("/") {
                name = repo.split("/").last().unwrap().into();
                repo_identifier = repo;
            } else {
                name = repo;
                repo_identifier = format!("{}/{}", repo_type.get_plural_name(), &name);
            }

            match clone_method {
                CloneMethod::Ssh => {
                    let username = config.gerrit_username
                        .clone()
                        .expect("Gerrit username not set in config");
                    (name, format!("ssh://{username}@gerrit.wikimedia.org:29418/mediawiki/{repo_identifier}"))
                },
                CloneMethod::Https => {
                    (name, format!("https://gerrit.wikimedia.org/r/{repo_identifier}.git"))
                }
            }
        },
        RepoOrigin::Github => {
            let url = match clone_method {
                CloneMethod::Ssh => format!("git@github.com:{repo}"),
                CloneMethod::Https => format!("https://github.com/{repo}.git"),
            };
            let mut name = repo.split("/").last().unwrap().to_string();
            let re = Regex::new(r"mediawiki-(?:extension|skin|service)s?-(.*)")
                .unwrap();
            if let Some(matched_name) = re.captures(&name).and_then(|c| c.get(1)) {
                name = matched_name.as_str().to_string()
            }
            (name, url)
        }
    }
}

fn clone(
    url: &str,
    name: &str,
    workdir: &Path,
    shallow: bool,
    branch: Option<&String>
) -> (ExitStatus, String, String) {
    let mut cmd = Command::new("git");
    cmd.args(["clone", url, name]);
    if shallow {
        cmd.arg("--depth=1");
    }
    if let Some(branch_name) = branch {
        cmd.args(["--branch", branch_name]);
    }
    cmd.current_dir(workdir);
    // TODO fix unwrap
    cmd.live_output().unwrap()
}
