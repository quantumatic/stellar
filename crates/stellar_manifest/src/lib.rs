//! # Manifest
//!
//! Every Stellar package has an associated manifest file, which contains metadata
//! that compiler needs to know to be able to compile it.
//!
//! Every package manifest is written in [TOML] format and consists of 2 parts:
//!
//! ```toml
//! [package]
//! name = "json"
//! version = "0.1.0"
//! author = "abs0luty"
//! license = "MIT"
//! repository = "https://github.com/abs0luty/json"
//! ...
//!
//! [dependencies]
//! hashmap = { version = "1.0.1", author = "quantumatic" }
//! http = { version = "1.0.0", author = "quantumatic" }
//! serialization_engine = { path = "../serialization_engine" }
//! ```
//!
//! The first part is a general information about the package, the second part is optional
//! and contains information about the dependencies of the current package.
//!
//! [TOML]: https://toml.io/en/v1.0.0

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/quantumatic/stellar/main/additional/icon/stellar.png",
    html_favicon_url = "https://raw.githubusercontent.com/quantumatic/stellar/main/additional/icon/stellar.png"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![warn(clippy::dbg_macro)]
#![warn(
    // rustc lint groups https://doc.rust-lang.org/rustc/lints/groups.html
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

/// Describes the package manifest, which contains information about the package.
///
/// See [crate level documentation] for more information.
///
/// [crate level documentation]: crate
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TomlManifest {
    /// The `[package]` section of the manifest.
    pub package: TomlPackage,
    /// The `[dependencies]` section of the manifest.
    pub dependencies: Option<BTreeMap<String, TomlDependency>>,
}

impl TomlManifest {
    /// Returns a new toml manifest struct with a given package.
    #[inline]
    #[must_use]
    pub const fn new(package: TomlPackage) -> Self {
        Self {
            package,
            dependencies: None,
        }
    }

    /// Returns a new toml manifest struct with given dependencies.
    #[inline]
    #[must_use]
    pub fn with_dependencies(
        mut self,
        dependencies: impl IntoIterator<Item = (impl Into<String>, TomlDependency)>,
    ) -> Self {
        self.dependencies = Some(
            dependencies
                .into_iter()
                .map(|(s, d)| (s.into(), d))
                .collect(),
        );
        self
    }
}

/// Represents data in the `[package]` section of the manifest.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TomlPackage {
    /// The name of the package.
    pub name: String,
    /// The latest version of the package.
    pub version: String,
    /// The authors of the package.
    pub description: Option<String>,
    /// The license of the package.
    pub license: Option<String>,
    /// Author of the package.
    pub author: Option<String>,
    /// Link to the repository of the package.
    pub repository: Option<String>,
    /// Keywords associated with the package.
    pub keywords: Option<Vec<String>>,
    /// Categories associated with the package.
    pub categories: Option<Vec<String>>,
}

impl TomlPackage {
    /// Returns a new toml package struct with a given name and version.
    /// Other fieds can be constructed using `with_*` methods.
    #[inline]
    #[must_use]
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: None,
            license: None,
            author: None,
            repository: None,
            keywords: None,
            categories: None,
        }
    }

    /// Builds a new toml package struct with a given description.
    #[inline]
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Builds a new toml package struct with a given license.
    #[inline]
    #[must_use]
    pub fn with_license(mut self, license: impl Into<String>) -> Self {
        self.license = Some(license.into());
        self
    }

    /// Builds a new toml package struct with a given author.
    #[inline]
    #[must_use]
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Builds a new toml package struct with a given repository.
    #[inline]
    #[must_use]
    pub fn with_repository(mut self, repository: impl Into<String>) -> Self {
        self.repository = Some(repository.into());
        self
    }

    /// Builds a new toml package struct with given keywords.
    #[inline]
    #[must_use]
    pub fn with_keywords(mut self, keywords: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.keywords = Some(keywords.into_iter().map(Into::into).collect());
        self
    }

    /// Builds a new toml package struct with given categories.
    #[inline]
    #[must_use]
    pub fn with_categories(
        mut self,
        categories: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.categories = Some(categories.into_iter().map(Into::into).collect());
        self
    }
}

/// Represents dependency (value part of the key-value pair in the `[dependencies]` section of the manifest).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct TomlDependency {
    /// The version of the dependency.
    pub version: Option<String>,

    /// The path to the local dependency's folder.
    pub path: Option<String>,

    /// The author of the dependency.
    pub author: Option<String>,
}

impl TomlDependency {
    /// Returns a new empty toml dependency struct.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds a new toml dependency struct with a given version.
    #[inline]
    #[must_use]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Builds a new toml dependency struct with a given path.s
    #[inline]
    #[must_use]
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Builds a new toml dependency struct with a given author.
    #[inline]
    #[must_use]
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }
}

/// # Errors
///
/// Error occurs when manifest format is not valid, or the structure itself is also not valid.
/// See [crate level documentation] for more information.
///
/// [crate level documentation]: crate
pub fn parse_manifest(source: impl AsRef<str>) -> Result<TomlManifest, String> {
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

fn parse_document(source: impl AsRef<str>) -> Result<Document, String> {
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
