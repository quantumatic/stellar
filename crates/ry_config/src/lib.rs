use std::{collections::HashMap, path::Path};

use ry_diagnostics::RyDiagnostic;

pub mod diagnostics;

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub project: ProjectMetadata,
    pub dependencies: HashMap<String, DependencySource>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectMetadata {
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub license: Option<String>,
    pub description: Option<String>,
    pub repository: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DependencySource {
    Local(Box<Path>),
    Remote {
        author: Option<String>,
        version: Option<String>,
    },
}

#[allow(clippy::ptr_arg)]
pub fn parse_config_from_source<S>(_source: S, _diagnostics: &mut Vec<RyDiagnostic>) {
    todo!()
}
