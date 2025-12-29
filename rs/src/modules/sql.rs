use std::process::Command;
use anyhow::Context;
use clap::Args;
use crate::config::MWUtilConfig;
use crate::exec::{create_db_command, ContainerSupport, DbCommandType, DbCommandUser};
use crate::modules::run;
use crate::modules::run::RunArgs;

#[derive(Args)]
pub struct SqlArgs {
    /// Execute the command as the root user in the container
    #[arg(short, long)]
    root: bool,

    /// Additional arguments to pass to the SQL command
    #[arg(trailing_var_arg = true)]
    extra_args: Vec<String>,
}

pub fn execute(config: &MWUtilConfig, args: SqlArgs) -> anyhow::Result<()> {
    if args.root {
        let status = create_db_command(config, DbCommandType::Query, DbCommandUser::Root)?
            .args(args.extra_args)
            .status()?;

        if !status.success() {
            anyhow::bail!("sql command failed with status: {}", status);
        }

        Ok(())
    } else {
        run::execute(config, RunArgs {
            script: "sql".to_string(),
            extra_args: args.extra_args,
        })
    }
}