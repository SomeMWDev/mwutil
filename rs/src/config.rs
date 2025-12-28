use std::{env, fs};
use std::path::{PathBuf};
use std::str::FromStr;
use serde::Deserialize;

const CONFIG_FILE_NAME: &str = ".mwutil.json";

#[derive(Debug, Deserialize)]
pub enum DBType {
    Mysql,
    Mariadb,
}

#[derive(Debug)]
pub struct ParseDBTypeError;

impl FromStr for DBType {
    type Err = ParseDBTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("mysql") {
            Ok(DBType::Mysql)
        } else if s.eq_ignore_ascii_case("mariadb") {
            Ok(DBType::Mariadb)
        } else {
            Err(ParseDBTypeError)
        }
    }
}

impl DBType {
    fn get_db_name(&self) -> &'static str {
        match self {
            DBType::Mysql => "mysql",
            DBType::Mariadb => "mariadb"
        }
    }

    fn get_container_name(&self) -> &'static str {
        match self {
            DBType::Mysql => "mysql",
            DBType::Mariadb => "mariadb"
        }
    }

    fn get_query_command(&self) -> &'static str {
        match self {
            DBType::Mysql => "mysql",
            DBType::Mariadb => "mariadb"
        }
    }

    fn get_dump_command(&self) -> &'static str {
        match self {
            DBType::Mysql => "mysqldump",
            DBType::Mariadb => "mariadb-dump"
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct MWUtilConfig {
    pub base_dir: PathBuf,
    pub config_dir: PathBuf,
    pub core_dir: PathBuf,
    pub dump_dir: PathBuf,
    pub db_type: DBType,
    pub mw_install_path: String,
    pub mw_branch: String,
}

#[derive(Debug)]
pub struct LoadMWUtilConfigError(pub &'static str);

pub fn load_mwutil_config() -> Result<MWUtilConfig, LoadMWUtilConfigError> {
    let base_dir = find_base_dir()?;

    let file = base_dir.join(CONFIG_FILE_NAME);
    let contents = fs::read_to_string(&file)
        .map_err(|_| LoadMWUtilConfigError("Failed to read config file"))?;
    let json_data: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|_| LoadMWUtilConfigError("Failed to parse config file as JSON"))?;

    fn get_dir(json_data: &serde_json::Value, base_dir: &PathBuf, key: &str, default: &str) -> PathBuf {
        base_dir.join(
            json_data
                .get(key)
                .and_then(|v| v.as_str())
                .unwrap_or(default)
        )
    }

    let config_dir = get_dir(&json_data, &base_dir, "configdir", "config");

    load_env(&config_dir);

    let db_type = DBType::from_str(
        env::var("MWC_DB_TYPE").unwrap_or(String::from("mariadb")).as_str()
    ).unwrap_or(DBType::Mariadb);
    let mw_install_path = env::var("MW_INSTALL_PATH")
        .unwrap_or(String::from("/var/www/html/w"));
    let mw_branch = env::var("MW_BRANCH")
        .unwrap_or(String::from("master"));

    Ok(MWUtilConfig {
        config_dir,
        core_dir: get_dir(&json_data, &base_dir, "coredir", "core"),
        dump_dir: get_dir(&json_data, &base_dir, "dumpdir", "dumps"),
        base_dir,
        db_type,
        mw_install_path,
        mw_branch
    })
}

fn load_env(config_dir: &PathBuf) {
    dotenv::from_path(config_dir.join(".env")).ok();
}

fn find_base_dir() -> Result<PathBuf, LoadMWUtilConfigError> {
    let mut current = env::current_dir()
        .map_err(|_| LoadMWUtilConfigError("Could not get current directory"))?;

    loop {
        let candidate = current.join(CONFIG_FILE_NAME);

        if candidate.is_file() {
            return Ok(current);
        }

        match current.parent() {
            None => break,
            Some(parent) => current = parent.to_path_buf()
        }
    }

    Err(LoadMWUtilConfigError("Did not find base dir with config file."))
}