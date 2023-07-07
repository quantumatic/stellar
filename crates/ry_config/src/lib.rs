use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};
use toml::Table;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigTable {
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub license: Option<String>,
    pub description: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub dependencies: Option<Table>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DependencyMetadata {
    pub source: DependencySource,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DependencySource {
    Repository { version: String },
    Local { path: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub license: Option<String>,
    pub description: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub dependencies: HashMap<String, DependencyMetadata>,
}

pub fn parse_config(_config_path: &Path) -> Config {
    todo!()
}

pub fn parse_config_from_source<S>(source: S) -> Result<ConfigTable, toml::de::Error>
where
    S: AsRef<str>,
{
    toml::from_str(source.as_ref())
}
