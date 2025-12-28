use crate::utils::container_completer;
use clap_complete::ArgValueCompleter;
use clap::Args;
use crate::config::MWUtilConfig;
use crate::exec::create_docker_compose_command;

#[derive(Args)]
pub struct RecreateArgs {
    /// The container to execute the command in
    #[arg(add = ArgValueCompleter::new(container_completer))]
    container: Option<String>,
}
pub fn execute(config: &MWUtilConfig, args: RecreateArgs) -> anyhow::Result<()> {
    let mut cmd = create_docker_compose_command(config);
    cmd.args(["up", "-d", "--force-recreate"]);
    if let Some(container) = args.container {
        cmd.arg(container);
    }
    cmd.status()
        .ok();

    Ok(())
}
