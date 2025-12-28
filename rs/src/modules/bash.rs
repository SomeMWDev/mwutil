use std::path::Path;
use std::process::Command;
use crate::utils::container_completer;
use clap_complete::{ArgValueCompleter};
use clap::{Args};
use crate::constants::MEDIAWIKI_CONTAINER;
use crate::exec::ContainerSupport;

#[derive(Args)]
pub struct BashArgs {
    /// The container to execute the command in
    #[arg(short, long, add = ArgValueCompleter::new(container_completer))]
    container: Option<String>,

    /// The folder in the container to execute the command in
    #[arg(short, long)]
    folder: Option<String>,

    /// Execute the command as the root user in the container
    #[arg(short, long)]
    root: bool,

    #[arg(trailing_var_arg = true)]
    command: Vec<String>,
}

pub fn execute(args: BashArgs) -> anyhow::Result<()> {
    let container = args.container.as_deref().unwrap_or(MEDIAWIKI_CONTAINER);
    let (program, cmd_args) = match args.command.clone().split_first() {
        Some((first, rest)) => (first.clone(), rest.to_vec()),
        None => ("bash".to_string(), vec![])
    };
    let mut cmd = Command::new(program);
    cmd.args(cmd_args);
    if let Some(workdir) = args.folder {
        cmd.current_dir(Path::new(&workdir));
    };

    let mut exec_options = vec![];
    if args.root {
        exec_options.push("-u".into());
        exec_options.push("root".into());
    }

    cmd.in_container(container, Some(exec_options))
        .status()
        .ok();

    Ok(())
}
