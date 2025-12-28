use std::process::Command;
use clap::Args;
use crate::config::MWUtilConfig;
use crate::constants::MEDIAWIKI_CONTAINER;
use crate::exec::ContainerSupport;
use crate::utils::get_core_version;

#[derive(Args, Default)]
pub struct RunArgs {
    /// The name of the maintenance script to run
    script: String,

    /// Additional arguments to pass to the script
    #[arg(trailing_var_arg = true)]
    extra_args: Vec<String>,
}

pub fn execute(config: MWUtilConfig, args: RunArgs) -> anyhow::Result<()> {
    let core_version = get_core_version(&config);
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
    
    cmd.in_container(MEDIAWIKI_CONTAINER, None)
        .status()
        .ok();
    Ok(())
}
