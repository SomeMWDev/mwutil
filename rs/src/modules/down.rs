use crate::utils::container_completer;
use clap_complete::ArgValueCompleter;
use clap::Args;
use crate::config::MWUtilConfig;
use crate::exec::create_docker_compose_command;

#[derive(Args)]
pub struct DownArgs {
    /// The container to stop
    #[arg(add = ArgValueCompleter::new(container_completer))]
    pub container: Option<String>,
}

pub fn execute(config: &MWUtilConfig, args: DownArgs) -> anyhow::Result<()> {
    let mut cmd = create_docker_compose_command(config);
    cmd.arg("down");
    if let Some(container) = args.container {
        cmd.arg(container);
    }
    cmd.status()?;

    Ok(())
}
