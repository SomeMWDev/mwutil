use crate::config::MWUtilConfig;
use crate::exec::{create_db_command, get_database_from_param, DbCommandType, DbCommandUser};
use crate::farm_config::FarmCommandArgs;
use crate::modules::run::RunArgs;
use crate::Modules;
use clap::Args;

#[derive(Args)]
pub struct SqlArgs {
    #[command(flatten)]
    pub farm_command_args: FarmCommandArgs,

    /// Execute the command as the root user in the container
    #[arg(short, long)]
    root: bool,

    /// Additional arguments to pass to the SQL command
    #[arg(trailing_var_arg = true)]
    extra_args: Vec<String>,
}

pub fn execute(config: &MWUtilConfig, args: SqlArgs) -> anyhow::Result<()> {
    if args.root {
        let status = create_db_command(
            config,
            DbCommandType::Query,
            DbCommandUser::Root,
            None,
            None,
            Some(get_database_from_param(args.farm_command_args.wiki))
        )?
            .args(args.extra_args)
            .status()?;

        if !status.success() {
            anyhow::bail!("sql command failed with status: {}", status);
        }

        Ok(())
    } else {
        Modules::Run(RunArgs {
            script: "sql".to_string(),
            extra_args: args.extra_args,
            farm_command_args: args.farm_command_args
        }).run(config)
    }
}