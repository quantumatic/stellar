use std::collections::{BTreeSet, HashMap};
use std::path::Path;

use serde::de::IntoDeserializer;
use serde::{Deserialize, Serialize};
use toml_edit::Document;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TomlManifest<'path> {
    pub project: TomlProject,
    #[serde(borrow)]
    pub dependencies: Option<HashMap<String, TomlDependency<'path>>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TomlProject {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub categories: Option<Vec<String>>,
    pub license: Option<String>,
    pub authors: Option<Vec<String>>,
    pub repository: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum TomlDependency<'path> {
    Simple(String),
    #[serde(borrow)]
    Detailed(DetailedTomlDependency<'path>),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DetailedTomlDependency<'path> {
    version: Option<String>,
    author: Option<String>,
    #[serde(borrow)]
    path: Option<&'path Path>,
}

pub fn parse_manifest<'path, S>(source: S) -> Result<TomlManifest<'path>, String>
where
    S: AsRef<str>,
{
    let toml = parse_document(source)?;

    let mut unused = BTreeSet::new();
    let manifest: TomlManifest =
        match serde_ignored::deserialize(toml.into_deserializer(), |path| {
            let mut key = String::new();
            stringify(&mut key, &path);
            unused.insert(key);
        }) {
            Ok(manifest) => manifest,
            Err(err) => return Err(format!("{}", err)),
        };

    Ok(manifest)
}

fn parse_document<S>(source: S) -> Result<Document, String>
where
    S: AsRef<str>,
{
    match source.as_ref().parse::<Document>() {
        Ok(table) => Ok(table),
        Err(err) => Err(format!("{}", err)),
    }
}

fn stringify(dst: &mut String, path: &serde_ignored::Path<'_>) {
    use serde_ignored::Path;

    match *path {
        Path::Root => {}
        Path::Seq { parent, index } => {
            stringify(dst, parent);
            if !dst.is_empty() {
                dst.push('.');
            }
            dst.push_str(&index.to_string());
        }
        Path::Map { parent, ref key } => {
            stringify(dst, parent);
            if !dst.is_empty() {
                dst.push('.');
            }
            dst.push_str(key);
        }
        Path::Some { parent }
        | Path::NewtypeVariant { parent }
        | Path::NewtypeStruct { parent } => stringify(dst, parent),
    }
}
