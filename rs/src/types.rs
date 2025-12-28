use std::fmt::Display;
use std::str::FromStr;
use clap::ValueEnum;

#[derive(Clone, Debug, PartialEq, ValueEnum)]
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
