use crate::config::MWUtilConfig;
use crate::exec::create_docker_compose_command;

pub fn execute(config: &MWUtilConfig) -> anyhow::Result<()> {
    create_docker_compose_command(config)
        .args(["up", "-d"])
        .status()
        .ok();

    Ok(())
}
