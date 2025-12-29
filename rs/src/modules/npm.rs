use clap::Args;
use std::path::Path;
use std::process::Command;

#[derive(Args, Default)]
pub struct NpmArgs {
    /// The folder on the host to execute the command in
    #[arg(short, long)]
    pub folder: Option<String>,

    /// Additional arguments to pass to npm
    #[arg(trailing_var_arg = true)]
    pub extra_args: Vec<String>,
}

pub fn execute(args: NpmArgs) -> anyhow::Result<()> {
    let mut cmd = Command::new("npm");

    cmd.arg("install");
    cmd.args(args.extra_args);
    if let Some(workdir) = args.folder {
        cmd.current_dir(Path::new(&workdir));
    };

    cmd.status()?;
    Ok(())
}
