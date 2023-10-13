//! Defines everything needed for proper error reporting.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/quantumatic/stellar/main/additional/icon/stellar.png",
    html_favicon_url = "https://raw.githubusercontent.com/quantumatic/stellar/main/additional/icon/stellar.png"
)]
#![warn(missing_docs, clippy::dbg_macro)]
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
    clippy::too_many_arguments,
    clippy::needless_pass_by_value,
    clippy::similar_names
)]

pub mod diagnostic;
pub mod files;
#[macro_use]
mod macro_;
pub mod term;

use core::fmt;
use std::fmt::Display;

use stellar_filesystem::in_memory_file_storage::InMemoryFileStorage;
use stellar_fx_hash::FxHashSet;
use stellar_interner::PathId;

use crate::{
    diagnostic::{Diagnostic, Severity},
    term::{
        termcolor::{ColorChoice, StandardStream},
        Config,
    },
};

/// Stores basic information for reporting diagnostics.
#[derive(Debug)]
pub struct DiagnosticsEmitter {
    /// The stream in which diagnostics is reported into.
    writer: StandardStream,

    /// The config for diagnostics reporting.
    config: Config,

    /// The files that are involved in the diagnostics are temporarily stored here.
    file_storage: InMemoryFileStorage,
}

impl Default for DiagnosticsEmitter {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Global diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostics {
    /// Files that are involved in the diagnostics.
    pub files_involved: FxHashSet<PathId>,

    /// Diagnostics.
    pub diagnostics: Vec<Diagnostic>,
}

impl Default for Diagnostics {
    fn default() -> Self {
        Self::new()
    }
}

impl Diagnostics {
    /// Creates a new instance of [`Diagnostics`].
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            files_involved: FxHashSet::default(),
            diagnostics: vec![],
        }
    }

    /// Adds a diagnostic associated with some files.
    #[inline]
    pub fn add_diagnostic(&mut self, diagnostic: impl BuildDiagnostic) {
        let diagnostic = diagnostic.build();

        self.files_involved.extend(diagnostic.files_involved());
        self.diagnostics.push(diagnostic);
    }

    /// Returns `true` if diagnostics are fatal.
    #[inline]
    #[must_use]
    pub fn is_fatal(&self) -> bool {
        !self.is_ok()
    }

    /// Returns `true` if diagnostics are ok.
    #[inline]
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.diagnostics
            .iter()
            .all(|d| !is_fatal_severity(d.severity))
    }
}

/// Empty diagnostics manager (implements [`Files`]).
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, Hash)]
pub struct EmptyDiagnosticsManager;

/// Empty source file name (used for internal usage,
/// see [`EmptyDiagnosticsManager`] for more details).
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, Hash)]
pub struct EmptyName;

/// Empty source file source (used for internal usage,
/// see [`EmptyDiagnosticsManager`] for more details).
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, Hash)]
pub struct EmptySource;

impl Display for EmptyName {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl AsRef<str> for EmptySource {
    fn as_ref(&self) -> &str {
        ""
    }
}

impl DiagnosticsEmitter {
    /// Create a new [`DiagnosticsEmitter`] instance.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            writer: StandardStream::stderr(ColorChoice::Always),
            config: Config::default(),
            file_storage: InMemoryFileStorage::new(),
        }
    }

    /// Set the stream in which diagnostics is reported into.
    #[inline]
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // false-positive clippy lint
    pub fn with_diagnostics_writer(mut self, writer: StandardStream) -> Self {
        self.writer = writer;
        self
    }

    /// Set the config for diagnostics reporting.
    #[inline]
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // false-positive clippy lint
    pub fn with_diagnostics_config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    /// Emit diagnostics associated with a particular file. If the file
    /// cannot be read, stops executing (no panic, diagnostic is just ignored).
    ///
    /// # Panics
    /// * If the file with a given path does not exist.
    /// * If the file path id cannot be resolved in the path storage.
    #[inline]
    fn emit_diagnostic(&self, diagnostic: &Diagnostic) {
        term::emit(
            &mut self.writer.lock(),
            &self.config,
            &self.file_storage,
            diagnostic,
        )
        .unwrap();
    }

    /// Emit all of the single file diagnostics.
    #[allow(single_use_lifetimes)] // anonymous lifetimes in traits are unstable
    fn emit_diagnostics<'a>(&self, diagnostics: impl IntoIterator<Item = &'a Diagnostic>) {
        for diagnostic in diagnostics {
            self.emit_diagnostic(diagnostic);
        }
    }

    /// Add files involved in the diagnostics into the file storage (if needed).
    #[allow(single_use_lifetimes)] // anonymous lifetimes in traits are unstable
    fn initialize_file_storage<'a>(
        &mut self,
        files_involved: impl IntoIterator<Item = &'a PathId>,
    ) {
        for filepath in files_involved {
            self.file_storage.read_and_add_file_or_panic(*filepath);
        }
    }

    /// Emit global diagnostics.
    #[inline]
    pub fn emit_global_diagnostics(&mut self, global_diagnostics: &Diagnostics) {
        self.initialize_file_storage(&global_diagnostics.files_involved);
        self.emit_diagnostics(&global_diagnostics.diagnostics);
    }
}

/// General status of diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticsStatus {
    /// There are no fatal diagnostics.
    Ok,

    /// There are fatal diagnostics.
    Fatal,
}

/// Returns `true` if the given [`Severity`] is fatal.
#[inline]
#[must_use]
pub const fn is_fatal_severity(severity: Severity) -> bool {
    matches!(severity, Severity::Error | Severity::Bug)
}

/// Builds a diagnostic struct.
pub trait BuildDiagnostic {
    /// Convert [`self`] into [`Diagnostic`].
    #[must_use]
    fn build(self) -> Diagnostic;
}
