use clap::ValueEnum;
use regex::Regex;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use strum::EnumIter;
use crate::config::DBType;
use crate::constants::{MEDIAWIKI_CONTAINER, OPENSEARCH_CONTAINER};

#[derive(Clone, Debug, EnumIter, PartialEq, ValueEnum)]
pub enum RepoType {
    Extension,
    Skin,
    Service,
    Tool,
}

#[derive(Debug)]
pub struct ParseRepoTypeError;

impl FromStr for RepoType {
    type Err = ParseRepoTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "extension" => Ok(RepoType::Extension),
            "skin" => Ok(RepoType::Skin),
            "service" => Ok(RepoType::Service),
            "tool" => Ok(RepoType::Tool),
            _ => Err(ParseRepoTypeError),
        }
    }
}

impl RepoType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RepoType::Extension => "extension",
            RepoType::Skin => "skin",
            RepoType::Service => "service",
            RepoType::Tool => "tool",
        }
    }

    pub fn get_plural_name(&self) -> String {
        self.as_str().to_owned() + "s"
    }
}

#[derive(Clone, Debug, PartialEq, ValueEnum)]
pub enum RepoOrigin {
    Gerrit,
    Github,
}

#[derive(Clone, Debug, PartialEq, ValueEnum)]
pub enum CloneMethod {
    Ssh,
    Https,
}

impl Display for CloneMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CloneMethod::Ssh => write!(f, "ssh"),
            CloneMethod::Https => write!(f, "https"),
        }
    }
}

#[derive(Debug)]
pub struct MWVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
    pub suffix: Option<String>,
}

impl MWVersion {
    pub fn parse(string: &str) -> Option<Self> {
        let re = Regex::new(r"(\d+)\.(\d+)\.(\d+)(?:-([a-zA-Z0-9]+))?")
            .unwrap();

        let captures = re.captures(string)?;

        let major: u8 = captures.get(1)?.as_str().parse().ok()?;
        let minor: u8 = captures.get(2)?.as_str().parse().ok()?;
        let patch: u8 = captures.get(3)?.as_str().parse().ok()?;
        let suffix: Option<String> = captures.get(4).map(|m| m.as_str().to_string());

        Some(Self {
            major,
            minor,
            patch,
            suffix
        })
    }
}

impl Display for MWVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let suffix = self.suffix
            .as_deref()
            .map(|s| format!("-{}", s))
            .unwrap_or_default();
        write!(f, "{}.{}.{}{}", self.major, self.minor, self.patch, suffix)
    }
}

pub enum Container {
    Database(DBType),
    MediaWiki,
    OpenSearch,
    Other(String),
}

impl Display for Container {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Container::Database(db_type) => db_type.get_container_name(),
            Container::MediaWiki => MEDIAWIKI_CONTAINER,
            Container::OpenSearch => OPENSEARCH_CONTAINER,
            Container::Other(name) => name,
        })
    }
}
