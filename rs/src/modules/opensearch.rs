use crate::config::{disable_profile, enable_profile, MWUtilConfig};
use crate::constants::{OPENSEARCH_CONTAINER, OPENSEARCH_PROFILE};
use crate::exec::ContainerSupport;
use crate::modules::down::DownArgs;
use crate::modules::recreate::RecreateArgs;
use crate::modules::run::RunArgs;
use crate::modules::{down, recreate, run};
use crate::types::Container;
use crate::utils::SpinnerSequence;
use clap::{Args, Subcommand};
use console::style;
use std::process::Command;

#[derive(Args)]
pub struct OpenSearchArgs {
    #[command(subcommand)]
    command: OpenSearchCommand,
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
    down::execute(config, DownArgs { container: Some(OPENSEARCH_CONTAINER.into()) })?;
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
    recreate::execute(config, RecreateArgs { container: Some(OPENSEARCH_CONTAINER.into()) })?;
    enable_profile(&mut config.clone(), profile)
}

fn reset(config: &MWUtilConfig) -> anyhow::Result<()> {
    let mut spinner = SpinnerSequence::new(2, "Resetting index");
    let mut cmd = Command::new("curl");
    cmd.args(["-X", "DELETE", "localhost:9200/_all"]);
    cmd.in_container(config, Container::OpenSearch, None)
        .status()?;

    spinner.next("Re-indexing wiki pages");
    run::execute(config, RunArgs {
        script: "CirrusSearch:UpdateSearchIndexConfig".into(),
        extra_args: vec!["--startOver".into()],
    })?;
    run::execute(config, RunArgs {
        script: "CirrusSearch:ForceSearchIndex".into(),
        ..Default::default()
    })?;
    spinner.finish();
    Ok(())
}
