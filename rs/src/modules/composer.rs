use std::path::Path;
use std::process::Command;
use clap::Args;
use crate::config::MWUtilConfig;
use crate::constants::MEDIAWIKI_CONTAINER;
use crate::exec::ContainerSupport;

#[derive(Args, Default)]
pub struct ComposerArgs {
    /// The folder in the container to execute the command in
    #[arg(short, long)]
    folder: Option<String>,

    /// Additional arguments to pass to composer
    #[arg(trailing_var_arg = true)]
    extra_args: Vec<String>,
}

pub fn execute(config: &MWUtilConfig, args: ComposerArgs) -> anyhow::Result<()> {
    let mut cmd = Command::new("composer");

    cmd.arg("update");
    cmd.args(args.extra_args);
    if let Some(workdir) = args.folder {
        cmd.current_dir(Path::new(&workdir));
    };

    cmd.in_container(config, MEDIAWIKI_CONTAINER, None)
        .status()
        .ok();

    Ok(())
}
