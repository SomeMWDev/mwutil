use crate::Modules;
use crate::config::MWUtilConfig;
use crate::exec::ExecutionHelper;
use crate::modules::composer::ComposerArgs;
use crate::modules::npm::NpmArgs;
use crate::types::{Container, ExecutionEnvironment};
use anyhow::bail;
use clap::{Args, ValueEnum};
use std::path::PathBuf;
use std::process::{Command, ExitStatus};
use std::str::FromStr;

#[derive(Clone, ValueEnum)]
enum LintType {
    Eslint,
    Phan,
    Phpcs,
}

#[derive(Args)]
pub struct LintArgs {
    /// The linter to execute
    lint_type: LintType,

    /// The folder to run the linter in
    folder: Option<String>,

    /// Whether to fix lint errors if possible
    #[arg(long)]
    fix: bool,
}

impl LintType {
    fn base_command(&self) -> Command {
        Command::new(match self {
            LintType::Eslint => "npm",
            LintType::Phan | LintType::Phpcs => "composer",
        })
    }

    fn get_lint_args(&self) -> &[&str] {
        match self {
            LintType::Eslint => &["run", "lint"],
            LintType::Phan => &["run", "phan"],
            LintType::Phpcs => &["run", "test"],
        }
    }

    fn get_execution_environment(&self) -> ExecutionEnvironment  {
        match self {
            LintType::Eslint => ExecutionEnvironment::Fresh,
            LintType::Phan | LintType::Phpcs => ExecutionEnvironment::MWContainer(Container::MediaWiki),
        }
    }

    fn execute(&self, config: &MWUtilConfig, folder: Option<String>, args: &[&str]) -> anyhow::Result<ExitStatus> {
        let mut cmd = self.base_command();
        cmd.args(args);
        cmd.env("XDEBUG_MODE", "off");

        if let Some(folder) = folder.clone() {
            cmd.current_dir(PathBuf::from_str(folder.as_str())?);
        }

        cmd = cmd.in_environment(config, self.get_execution_environment(), None)?;

        let status = cmd.status()?;
        Ok(status)
    }

    fn fix(&self, config: &MWUtilConfig, folder: Option<String>) -> anyhow::Result<ExitStatus> {
        self.execute(config, folder, match self {
            LintType::Eslint => &["run", "lint:fix:js"],
            LintType::Phan => bail!("--fix is not implemented for phan!"),
            LintType::Phpcs => &["run", "fix"],
        })
    }
}

pub fn execute(config: &MWUtilConfig, args: LintArgs, update: bool) -> anyhow::Result<()> {
    let status = args.lint_type.execute(
        config,
        args.folder.clone(),
        args.lint_type.get_lint_args()
    )?;

    if update && !status.success() && status.code() == Some(127) {
        println!("Failed to lint. Attempting to update dependencies...");
        match args.lint_type {
            LintType::Eslint => Modules::Npm(NpmArgs {
                folder: args.folder.clone(),
                ..Default::default()
            }).run(config)?,
            LintType::Phan | LintType::Phpcs => Modules::Composer(ComposerArgs {
                folder: args.folder.clone(),
                ..Default::default()
            }).run(config)?,
        }

        return execute(config, args, false);
    }

    if args.fix {
        args.lint_type.fix(config, args.folder)?;
    }

    Ok(())
}
