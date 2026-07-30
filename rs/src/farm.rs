use clap_complete::ArgValueCompleter;
use crate::config::{find_base_dir, MWUtilConfig};
use anyhow::{anyhow, Context};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use clap::Args;
use clap_complete::CompletionCandidate;

pub enum Wiki {
    ByName(String),
    Central,
}

pub fn get_wiki_from_param(param: Option<String>) -> Wiki {
    match param {
        None => Wiki::Central,
        Some(name) => Wiki::ByName(name)
    }
}

pub fn get_db_name(config: &MWUtilConfig, wiki: Wiki) -> anyhow::Result<String> {
    match wiki {
        Wiki::ByName(name) => Ok(name),
        Wiki::Central => {
            match load_farm_config(config)? {
                None => Ok(config.mw_database.clone().ok_or_else(|| anyhow!("MW DB must be set"))?),
                Some(farm) => Ok(farm.central_wiki),
            }
        }
    }
}

pub struct FarmConfig {
    pub central_wiki: String,
    pub wikis: Vec<String>,
}

pub fn load_farm_config(config: &MWUtilConfig) -> anyhow::Result<Option<FarmConfig>> {
    load_farm_config_fast(config.base_dir.clone())
}

// This does the same as load_farm_config(), but without requiring a full config object.
pub fn load_farm_config_fast(base_dir: PathBuf) -> anyhow::Result<Option<FarmConfig>> {
    let file = base_dir.join("config").join("farm-config.json");
    let exists = fs::exists(&file)?;
    if !exists {
        return Ok(None)
    }
    let json = fs::read_to_string(file)
        .context("Failed to read farm-config.json!")?;
    let data: Value = serde_json::from_str(&json)
        .context("Failed to parse farm-config.json as JSON!")?;
    if let Some(wikis) = data.get("wikis").and_then(Value::as_object) {
        let wiki_names: Vec<String> = wikis.keys().cloned().collect();

        let central_wiki = data["centralWiki"].as_str()
            .ok_or_else(|| anyhow!("Failed to retrieve central wiki!"))?;

        return Ok(Some(FarmConfig {
            central_wiki: central_wiki.into(),
            wikis: wiki_names
        }))
    }
    Err(anyhow!("Failed to parse farm config."))
}

#[derive(Args, Default)]
pub struct FarmCommandArgs {
    /// The database name of the wiki that will be used
    #[arg(add = ArgValueCompleter::new(wiki_completer))]
    pub wiki: Option<String>,
}

fn wiki_completer(_current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    let Some(base_dir) = find_base_dir() else {
        return vec![];
    };
    let Ok(farm_config) = load_farm_config_fast(base_dir) else {
        return vec![];
    };

    farm_config.unwrap().wikis.iter().map(|wiki|CompletionCandidate::new(wiki)).collect()
}

pub trait FarmSupport {
    fn on_wiki(&mut self, config: &MWUtilConfig, wiki: Wiki) -> anyhow::Result<()>;
}

impl FarmSupport for Command {
    fn on_wiki(&mut self, config: &MWUtilConfig, wiki: Wiki) -> anyhow::Result<()> {
        self.args(vec!["--wiki", &get_db_name(config, wiki)?]);
        Ok(())
    }
}

