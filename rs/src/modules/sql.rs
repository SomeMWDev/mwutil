use std::process::Command;
use clap::Args;
use crate::config::MWUtilConfig;
use crate::exec::ContainerSupport;
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
        let mut cmd = Command::new(config.db_type.get_query_command());
        let root_password = config.db_root_password.as_ref().ok_or_else(|| anyhow::anyhow!("Database root password is not set in the configuration"))?;
        let database = config.mw_database.as_ref().ok_or_else(|| anyhow::anyhow!("MediaWiki database name is not set in the configuration"))?;
        cmd.args([database, "-uroot", &format!("-p{root_password}")]);
        cmd.args(args.extra_args);
        cmd.in_container(config, config.db_type.get_container_name(), None)
            .status()?;
        Ok(())
    } else {
        run::execute(config, RunArgs {
            script: "sql".to_string(),
            extra_args: args.extra_args,
        })
    }
}