use std::fs;
use clap::Args;
use std::path::PathBuf;
use std::sync::mpsc;
use anyhow::{bail, Context};
use cli_clipboard::{ClipboardContext, ClipboardProvider};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use notify::event::{AccessKind, AccessMode};

#[derive(Args, Default)]
pub struct WatchArgs {
    /// The file to watch
    pub file: PathBuf,
}

pub fn execute(args: WatchArgs) -> anyhow::Result<()> {
    let file = args.file;
    if !file.is_file() {
        bail!("The provided path is not a file!");
    }

    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(file.as_path(), RecursiveMode::NonRecursive)?;

    let mut ctx = ClipboardContext::new()
        .expect("Failed to create clipboard context!");

    for res in rx {
        match res {
            Ok(event) => {
                if let EventKind::Access(AccessKind::Close(AccessMode::Write)) = event.kind {
                    let contents = fs::read_to_string(&file)
                        .context("Failed to read updated file!")?;
                    let len = contents.len();
                    ctx.set_contents(contents)
                        .expect("Failed to copy to clipboard!");
                    println!("Copied {} bytes to clipboard!", len);
                }
            },
            Err(e) => println!("Error: {:?}", e),
        }
    }

    Ok(())
}
