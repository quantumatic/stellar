//! # Manifest
//!
//! Every Ry project has an associated manifest file, which contains metadata
//! that compiler needs to know to be able to compile it.
//!
//! Every project manifest is written in [TOML] format and consists of 2 parts:
//!
//! ```toml
//! [project]
//! name = "json"
//! version = "0.1.0"
//! author = "abs0luty"
//! license = "MIT"
//! repository = "https://github.com/abs0luty/json"
//! ...
//!
//! [dependencies]
//! hashmap = "1.0.1"
//! btreemap = { version = "1.0.0" }
//! serialization_engine = { path = "../serialization_engine" }
//! ```
//!
//! The first part is a general information about the project, the second part is optional
//! and contains information about the dependencies of the current project.
//!
//! [TOML]: https://toml.io/en/v1.0.0

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png",
    html_favicon_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![warn(clippy::dbg_macro)]
#![deny(
    // rustc lint groups https://doc.rust-lang.org/rustc/lints/groups.html
    warnings,
    future_incompatible,
    let_underscore,
    nonstandard_style,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    unused,
    // rustc allowed-by-default lints https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    non_ascii_idents,
    noop_method_call,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_op_in_unsafe_fn,
    unused_crate_dependencies,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_tuple_struct_fields,
    variant_size_differences,
    // rustdoc lints https://doc.rust-lang.org/rustdoc/lints.html
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_doc_tests,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::bare_urls,
    // clippy categories https://doc.rust-lang.org/clippy/
    clippy::all,
    clippy::correctness,
    clippy::suspicious,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::pedantic,
    clippy::nursery,
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::too_many_lines,
    clippy::option_if_let_else,
    clippy::redundant_pub_crate,
    clippy::unnested_or_patterns
)]

use std::collections::BTreeMap;

use serde::{de::IntoDeserializer, Deserialize, Serialize};
use toml as _;
use toml_edit::Document;

/// Describes the project manifest, which contains information about the project.
///
/// See [crate level documentation] for more information.
///
/// [crate level documentation]: crate
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TomlManifest {
    /// The `[project]` section of the manifest.
    pub project: TomlProject,
    /// The `[dependencies]` section of the manifest.
    pub dependencies: Option<BTreeMap<String, TomlDependency>>,
}

/// Represents data in the `[project]` section of the manifest.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TomlProject {
    /// The name of the project.
    pub name: String,
    /// The latest version of the project.
    pub version: String,
    /// The authors of the project.
    pub description: Option<String>,
    /// The license of the project.
    pub license: Option<String>,
    /// Author of the project.
    pub author: Option<String>,
    /// Link to the repository of the project.
    pub repository: Option<String>,
    /// Keywords associated with the project.
    pub keywords: Option<Vec<String>>,
    /// Categories associated with the project.
    pub categories: Option<Vec<String>>,
}

/// Represents dependency (value part of the key-value pair in the `[dependencies]` section of the manifest).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum TomlDependency {
    /// In the simple format, only a version is specified, eg.
    ///
    /// ```toml
    /// package = "<version>"
    /// ```
    Simple(String),
    /// The simple format is equivalent to a detailed dependency
    /// specifying only a version, eg.
    ///
    /// ```toml
    /// package = { version = "<version>" }
    /// ```
    Detailed(DetailedTomlDependency),
}

/// Detailed dependency information.
///
/// See [`TomlDependency::Detailed`] for more details.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct DetailedTomlDependency {
    /// The version of the dependency.
    pub version: Option<String>,
    pub path: Option<String>,
}

/// # Errors
///
/// Error occurs when manifest format is not valid, or the structure itself is also not valid.
/// See [crate level documentation] for more information.
///
/// [crate level documentation]: crate
pub fn parse_manifest<S>(source: S) -> Result<TomlManifest, String>
where
    S: AsRef<str>,
{
    let toml = parse_document(source)?;

    let manifest: TomlManifest =
        match serde_ignored::deserialize(toml.into_deserializer(), |path| {
            let mut key = String::new();
            stringify(&mut key, &path);
        }) {
            Ok(manifest) => manifest,
            Err(err) => return Err(format!("{err}")),
        };

    Ok(manifest)
}

#[inline]
fn parse_document<S>(source: S) -> Result<Document, String>
where
    S: AsRef<str>,
{
    match source.as_ref().parse::<Document>() {
        Ok(table) => Ok(table),
        Err(err) => Err(format!("{err}")),
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
