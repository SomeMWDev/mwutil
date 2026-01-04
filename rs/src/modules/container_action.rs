use clap_complete::ArgValueCompleter;
use clap::Args;
use crate::config::MWUtilConfig;
use crate::exec::create_docker_compose_command;
use crate::utils::container_completer;

#[derive(Args, Default)]
pub struct ContainerActionArgs {
    /// The container to perform the action on
    #[arg(add = ArgValueCompleter::new(container_completer))]
    pub container: Option<String>,
}

fn execute(config: &MWUtilConfig, args: ContainerActionArgs, compose_args: &[&str]) -> anyhow::Result<()> {
    let mut cmd = create_docker_compose_command(config);
    cmd.args(compose_args);
    if let Some(container) = args.container {
        cmd.arg(container);
    }
    cmd.status()?;
    Ok(())
}

pub fn recreate(config: &MWUtilConfig, args: ContainerActionArgs) -> anyhow::Result<()> {
    execute(
        config,
        args,
        &["up", "-d", "--force-recreate"]
    )
}

pub fn up(config: &MWUtilConfig, args: ContainerActionArgs) -> anyhow::Result<()> {
    execute(
        config,
        args,
        &["up", "-d"]
    )
}

pub fn down(config: &MWUtilConfig, args: ContainerActionArgs) -> anyhow::Result<()> {
    execute(
        config,
        args,
        &["down"]
    )
}
