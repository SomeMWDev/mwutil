use crate::constants::CONFIG_FILE_NAME;
use anyhow::{anyhow, Context};
use clap::ValueEnum;
use regex::Regex;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::{env, fs};

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
    pub mw_server: Option<String>,
    pub mw_script_path: Option<String>,
    pub mw_language: Option<String>,
    pub mw_user: Option<String>,
    pub mw_password: Option<String>,

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
    let config_dir = base_dir.join("config");

    load_env(&config_dir);

    let db_type = DBType::parse(
        env::var("MWC_DB_TYPE")
            .unwrap_or(String::from("mariadb"))
            .as_str()
    ).unwrap_or(DBType::Mariadb);
    let mw_install_path = env::var("MW_INSTALL_PATH")
        .unwrap_or(String::from("/var/www/html/w"));
    let mw_branch = env::var("MW_BRANCH")
        .unwrap_or(String::from("master"));

    let compose_profiles: Vec<String> = env::var("COMPOSE_PROFILES")
        .context("The COMPOSE_PROFILES env variable is required")?
        .split(",")
        .map(String::from)
        .collect();

    Ok(MWUtilConfig {
        config_dir,
        core_dir: base_dir.join("core"),
        dump_dir: base_dir.join("dumps"),
        base_dir,

        db_type,
        db_user: env::var("MWC_DB_USER").ok(),
        db_password: env::var("MWC_DB_PASSWORD").ok(),
        db_root_password: env::var("MWC_DB_ROOT_PASSWORD").ok(),
        mw_database: env::var("MWC_DB_DATABASE").ok(),
        mw_install_path,
        mw_branch,
        mw_server: env::var("MW_SERVER").ok(),
        mw_script_path: env::var("MW_SCRIPT_PATH").ok(),
        mw_language: env::var("MW_LANG").ok(),
        mw_user: env::var("MEDIAWIKI_USER").ok(),
        mw_password: env::var("MEDIAWIKI_PASSWORD").ok(),

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
    let new_line = format!("{}={}", var, val);
    let output = if re.is_match(&contents) {
        re.replace_all(&contents, &new_line).into_owned()
    } else {
        format!("{}\n{}", contents.trim_end(), new_line)
    };
    fs::write(env_file, output)
        .context("Failed to write to .env file!")?;
    Ok(())
}

pub fn update_profiles(config: &mut MWUtilConfig, profiles: &[String]) -> anyhow::Result<()> {
    config.compose_profiles = profiles.to_vec();
    update_env_var(config, "COMPOSE_PROFILES", profiles.join(",").as_str())
}

pub fn enable_profile(config: &mut MWUtilConfig, profile: String) -> anyhow::Result<()> {
    if config.compose_profiles.contains(&profile) {
        return Ok(())
    }
    let mut new_profiles = config.compose_profiles.clone();
    new_profiles.push(profile);
    update_profiles(config, new_profiles.as_slice())
}

pub fn disable_profile(config: &mut MWUtilConfig, profile: &str) -> anyhow::Result<()> {
    let mut new_profiles = config.compose_profiles.clone();
    new_profiles.retain(|p| p != profile);
    update_profiles(config, new_profiles.as_slice())
}
