use crate::config::{disable_profile, enable_profile, MWUtilConfig};
use crate::constants::{OPENSEARCH_CONTAINER, OPENSEARCH_PROFILE};
use crate::exec::ContainerSupport;
use crate::modules::container_action::ContainerActionArgs;
use crate::modules::run::RunArgs;
use crate::types::Container;
use crate::utils::SpinnerSequence;
use crate::Modules;
use clap::{Args, Subcommand};
use console::style;
use std::process::Command;

#[derive(Args)]
pub struct OpenSearchArgs {
    #[command(subcommand)]
    pub command: OpenSearchCommand,
}

#[derive(Subcommand)]
pub enum OpenSearchCommand {
    /// Disables the OpenSearch container and profile
    Disable,
    /// Enables the OpenSearch container and profile
    Enable,
    /// Resets the OpenSearch index and reindexes the wiki
    Reset,
}

pub fn execute(config: &MWUtilConfig, args: OpenSearchArgs) -> anyhow::Result<()> {
    match args.command {
        OpenSearchCommand::Disable => disable(config),
        OpenSearchCommand::Enable => enable(config),
        OpenSearchCommand::Reset => reset(config),
    }
}

fn disable(config: &MWUtilConfig) -> anyhow::Result<()> {
    if !config.compose_profiles.contains(&String::from(OPENSEARCH_PROFILE)) {
        anyhow::bail!(
            "OpenSearch is {}!",
            style("already disabled").red(),
        )
    }
    Modules::Down(ContainerActionArgs {
        container: Some(OPENSEARCH_CONTAINER.into())
    }).run(config)?;
    disable_profile(&mut config.clone(), OPENSEARCH_PROFILE)
}

fn enable(config: &MWUtilConfig) -> anyhow::Result<()> {
    let profile = String::from(OPENSEARCH_PROFILE);
    if config.compose_profiles.contains(&profile) {
        anyhow::bail!(
            "OpenSearch is {}!",
            style("already enabled").red(),
        )
    }
    Modules::Up(ContainerActionArgs {
        container: Some(OPENSEARCH_CONTAINER.into())
    }).run(config)?;
    enable_profile(&mut config.clone(), profile)
}

fn reset(config: &MWUtilConfig) -> anyhow::Result<()> {
    let mut spinner = SpinnerSequence::new(2, "Resetting index");
    let mut cmd = Command::new("curl");
    cmd.args(["-X", "DELETE", "localhost:9200/_all"]);
    cmd.in_container(config, Container::OpenSearch, None)?
        .status()?;

    spinner.next("Re-indexing wiki pages");
    Modules::Run(RunArgs {
        script: "CirrusSearch:UpdateSearchIndexConfig".into(),
        extra_args: vec!["--startOver".into()],
    }).run(config)?;
    Modules::Run(RunArgs {
        script: "CirrusSearch:ForceSearchIndex".into(),
        ..Default::default()
    }).run(config)?;
    spinner.finish();
    Ok(())
}
