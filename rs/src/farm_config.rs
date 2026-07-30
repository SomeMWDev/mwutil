use std::fs;
use anyhow::{anyhow, Context};
use serde_json::Value;
use crate::config::find_base_dir;

pub struct FarmConfig {
    pub wikis: Vec<String>,
}

pub fn load_farm_config() -> anyhow::Result<Option<FarmConfig>> {
    let base_dir = find_base_dir()
        .ok_or_else(|| anyhow!("Failed to find basedir"))?;
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
        return Ok(Some(FarmConfig {
            wikis: wiki_names
        }))
    }
    Err(anyhow!("Failed to parse farm config."))
}
