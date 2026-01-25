use crate::config::MWUtilConfig;
use crate::exec::ContainerSupport;
use crate::types::Container;
use clap::Args;
use std::path::Path;
use std::process::Command;

#[derive(Args, Default)]
pub struct ComposerArgs {
    /// The folder in the container to execute the command in
    #[arg(short, long)]
    pub folder: Option<String>,

    /// Additional arguments to pass to composer
    #[arg(trailing_var_arg = true)]
    pub extra_args: Vec<String>,
}

pub fn execute(config: &MWUtilConfig, args: ComposerArgs) -> anyhow::Result<()> {
    let mut cmd = Command::new("composer");

    cmd.arg("update");
    cmd.args(args.extra_args);
    if let Some(workdir) = args.folder {
        cmd.current_dir(Path::new(&workdir));
    };

    cmd.in_container(config, Container::MediaWiki, None)?
        .status()
        .ok();

    Ok(())
}
