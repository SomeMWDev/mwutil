use std::fs;
use anyhow::{anyhow, bail};
use clap::{Args, ValueEnum};
use crate::config::MWUtilConfig;
use crate::constants::MEDIAWIKI_CONTAINER;
use crate::modules::opensearch::{OpenSearchArgs, OpenSearchCommand};
use crate::modules::run::RunArgs;
use crate::{Modules};
use crate::modules::db;
use crate::utils::SpinnerSequence;

// only supports top-level files in images/ !
const EXCLUDED_UPLOADS: [&str; 2] = ["README", ".htaccess"];

#[derive(Clone, PartialEq, ValueEnum)]
pub enum ResetActions {
    Database,
    OpenSearch,
    Uploads,
}

#[derive(Args)]
pub struct ResetArgs {
    /// The actions to perform
    actions: Vec<ResetActions>
}

pub fn execute(config: &MWUtilConfig, args: ResetArgs) -> anyhow::Result<()> {
    let actions = args.actions;
    // not using SpinnerSequence::new here so we can init the text dynamically
    let mut spinner = SpinnerSequence {
        cur: 0,
        max: actions.len() as u8,
        last: None,
    };
    if actions.contains(&ResetActions::Uploads) {
        spinner.next("Resetting uploads");
        reset_uploads(config)?;
    }
    if actions.contains(&ResetActions::Database) {
        spinner.next("Resetting database");
        reset_database(config)?;
    }
    if actions.contains(&ResetActions::OpenSearch) {
        spinner.next("Resetting OpenSearch");
        Modules::OpenSearch(OpenSearchArgs {
            command: OpenSearchCommand::Reset,
        }).run(config)?;
    }
    Ok(())
}

pub fn reset_uploads(config: &MWUtilConfig) -> anyhow::Result<()> {
    let upload_dir = config.core_dir.join("images");
    if !upload_dir.exists() {
        bail!("Upload directory does not exist!");
    }
    if !upload_dir.is_dir() {
        bail!("Upload directory is not a directory!");
    }
    if let Ok(files) = fs::read_dir(upload_dir) {
        for entry in files.flatten() {
            if !EXCLUDED_UPLOADS.contains(&entry.file_name().to_string_lossy().as_ref()) {
                let path = entry.path();
                println!("Removing {}", entry.file_name().to_string_lossy());
                if path.is_dir() {
                    fs::remove_dir_all(path)?;
                } else {
                    fs::remove_file(path)?;
                }
            }
        }
    }
    Ok(())
}

pub fn reset_database(config: &MWUtilConfig) -> anyhow::Result<()> {
    db::drop_mw_database(config)?;

    let local_settings = config.core_dir.join("LocalSettings.php");
    let local_settings_tmp = config.core_dir.join("LocalSettings.temp.php");
    fs::rename(&local_settings, &local_settings_tmp)?;

    let install_args = vec![
        format!("--dbname={}", config.mw_database.clone().ok_or_else(|| anyhow!("MW Database not set!"))?),
        format!("--dbuser={}", config.db_user.clone().ok_or_else(|| anyhow!("DB User not set!"))?),
        format!("--dbpass={}", config.db_password.clone().ok_or_else(|| anyhow!("DB Password not set!"))?),
        format!("--dbserver={}", config.db_type.clone().get_container_name()),
        format!("--server={}", config.mw_server.clone().ok_or_else(|| anyhow!("MW Server not set!"))?),
        format!("--scriptpath={}", config.mw_script_path.clone().ok_or_else(|| anyhow!("MW Script Path not set!"))?),
        format!("--lang={}", config.mw_language.clone().ok_or_else(|| anyhow!("MW Language not set!"))?),
        format!("--pass={}", config.mw_password.clone().ok_or_else(|| anyhow!("MW Password not set!"))?),
        MEDIAWIKI_CONTAINER.to_string(),
        config.mw_user.clone().ok_or_else(|| anyhow!("MW User not set!"))?,
    ];

    let result = Modules::Run(RunArgs {
        script: "install".to_string(),
        extra_args: install_args,
    }).run(config);

    fs::rename(local_settings_tmp, local_settings)?;

    result?;

    Modules::Update.run(config)?;
    Modules::Recreate(Default::default()).run(config)?;

    Ok(())
}
