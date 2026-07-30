use crate::config::MWUtilConfig;
use anyhow::{anyhow, Context};
use serde_json::Value;
use std::fs;

pub enum Wiki {
    ByName(String),
    Central,
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
    let file = config.base_dir.clone().join("config").join("farm-config.json");
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

        let central_wiki = data["central_wiki"].as_str()
            .ok_or_else(|| anyhow!("Failed to retrieve central wiki!"))?;

        return Ok(Some(FarmConfig {
            central_wiki: central_wiki.into(),
            wikis: wiki_names
        }))
    }
    Err(anyhow!("Failed to parse farm config."))
}
