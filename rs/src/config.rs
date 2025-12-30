use anyhow::{anyhow, Context};
use clap::ValueEnum;
use regex::Regex;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::{env, fs};

const CONFIG_FILE_NAME: &str = ".mwutil.json";

#[derive(Clone, Debug, PartialEq, ValueEnum)]
pub enum DBType {
    Mysql,
    Mariadb,
}

impl DBType {
    fn parse(s: &str) -> Option<Self> {
        if s.eq_ignore_ascii_case("mysql") {
            Some(DBType::Mysql)
        } else if s.eq_ignore_ascii_case("mariadb") {
            Some(DBType::Mariadb)
        } else {
            None
        }
    }

    pub fn all_values() -> Vec<Self> {
        vec![DBType::Mariadb, DBType::Mysql]
    }
}

impl Display for DBType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            DBType::Mysql => "MySQL",
            DBType::Mariadb => "MariaDB",
        })
    }
}

impl DBType {
    pub fn get_db_name(&self) -> &'static str {
        match self {
            DBType::Mysql => "mysql",
            DBType::Mariadb => "mariadb"
        }
    }

    pub fn get_container_name(&self) -> &'static str {
        match self {
            DBType::Mysql => "mysql",
            DBType::Mariadb => "mariadb"
        }
    }

    pub fn get_query_command(&self) -> &'static str {
        match self {
            DBType::Mysql => "mysql",
            DBType::Mariadb => "mariadb"
        }
    }

    pub fn get_dump_command(&self) -> &'static str {
        match self {
            DBType::Mysql => "mysqldump",
            DBType::Mariadb => "mariadb-dump"
        }
    }
}

#[derive(Clone, Debug)]
pub struct MWUtilConfig {
    pub base_dir: PathBuf,
    pub config_dir: PathBuf,
    pub core_dir: PathBuf,
    pub dump_dir: PathBuf,

    pub db_type: DBType,
    pub db_user: Option<String>,
    pub db_password: Option<String>,
    pub db_root_password: Option<String>,
    pub mw_database: Option<String>,
    pub mw_install_path: String,
    pub mw_branch: String,

    pub gerrit_username: Option<String>,
    pub git_email: Option<String>,
    pub git_username: Option<String>,
    pub security_patch_folder: Option<String>,

    pub compose_profiles: Vec<String>,

    pub debug: bool,
}

pub fn load_mwutil_config(debug: bool) -> anyhow::Result<MWUtilConfig> {
    let base_dir = find_base_dir()
        .ok_or_else(|| anyhow!("Failed to find basedir"))?;

    let file = base_dir.join(CONFIG_FILE_NAME);
    let contents = fs::read_to_string(&file)
        .context("Failed to read config file")?;
    let json_data: serde_json::Value = serde_json::from_str(&contents)
        .context("Failed to parse config file as JSON")?;

    fn get_dir(json_data: &serde_json::Value, base_dir: &Path, key: &str, default: &str) -> PathBuf {
        base_dir.join(
            json_data
                .get(key)
                .and_then(|v| v.as_str())
                .unwrap_or(default)
        )
    }

    // TODO we probably need to hardcode this instead and no longer load it from JSON
    // since we use hardcoded paths in autocompletion code to improve performance
    let config_dir = get_dir(&json_data, &base_dir, "configdir", "config");

    load_env(&config_dir);

    let db_type = DBType::parse(
        env::var("MWC_DB_TYPE").unwrap_or(String::from("mariadb")).as_str()
    ).unwrap_or(DBType::Mariadb);
    let mw_install_path = env::var("MW_INSTALL_PATH")
        .unwrap_or(String::from("/var/www/html/w"));
    let mw_branch = env::var("MW_BRANCH")
        .unwrap_or(String::from("master"));

    let compose_profiles: Vec<String> = env::var("COMPOSE_PROFILES")
        .context("COMPOSER_PROFILES env variable is required")?
        .split(",")
        .map(String::from)
        .collect();

    Ok(MWUtilConfig {
        config_dir,
        core_dir: get_dir(&json_data, &base_dir, "coredir", "core"),
        dump_dir: get_dir(&json_data, &base_dir, "dumpdir", "dumps"),
        base_dir,

        db_type,
        db_user: env::var("MWC_DB_USER").ok(),
        db_password: env::var("MWC_DB_PASSWORD").ok(),
        db_root_password: env::var("MWC_DB_ROOT_PASSWORD").ok(),
        mw_database: env::var("MWC_DB_DATABASE").ok(),
        mw_install_path,
        mw_branch,

        gerrit_username: env::var("GERRIT_USERNAME").ok(),
        git_email: env::var("GIT_EMAIL").ok(),
        git_username: env::var("GIT_USERNAME").ok(),
        security_patch_folder: env::var("SECURITY_PATCH_FOLDER").ok(),

        compose_profiles,

        debug,
    })
}

fn load_env(config_dir: &Path) {
    dotenv::from_path(config_dir.join(".env")).ok();
}

pub fn find_base_dir() -> Option<PathBuf> {
    let mut current = env::current_dir().ok()?;

    loop {
        let candidate = current.join(CONFIG_FILE_NAME);

        if candidate.is_file() {
            return Some(current);
        }

        match current.parent() {
            None => break,
            Some(parent) => current = parent.to_path_buf()
        }
    }

    None
}

pub fn update_env_var(config: &MWUtilConfig, var: &str, val: &str) -> anyhow::Result<()> {
    let env_file = config.config_dir.join(".env");
    let contents = fs::read_to_string(&env_file)
        .context("Failed to read .env file!")?;

    let re = Regex::new(&format!(r"(?m)^{}=.*$", regex::escape(var)))?;
    let output = re.replace_all(contents.as_str(), format!("{}={}", var, val));
    fs::write(env_file, output.as_ref())
        .context("Failed to write to .env file!")?;
    Ok(())
}

pub fn update_profiles(config: &mut MWUtilConfig, profiles: &[String]) -> anyhow::Result<()> {
    config.compose_profiles = profiles.to_vec();
    update_env_var(config, "COMPOSE_PROFILES", profiles.join(",").as_str())
}

pub fn enable_profile(config: &mut MWUtilConfig, profile: String) -> anyhow::Result<()> {
    let mut new_profiles = config.compose_profiles.clone();
    new_profiles.push(profile);
    update_profiles(config, new_profiles.as_slice())
}

pub fn disable_profile(config: &mut MWUtilConfig, profile: &str) -> anyhow::Result<()> {
    let mut new_profiles = config.compose_profiles.clone();
    new_profiles.retain(|p| p != profile);
    update_profiles(config, new_profiles.as_slice())
}
