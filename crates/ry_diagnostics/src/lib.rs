//! Defines everything needed for proper error reporting.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png",
    html_favicon_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png"
)]
#![warn(missing_docs, clippy::dbg_macro)]
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
    clippy::option_if_let_else
)]

use core::fmt;
use std::fmt::Display;

use codespan_reporting::{
    diagnostic::{Diagnostic, Severity},
    files::{self, Files},
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
        Config,
    },
};
use ry_filesystem::{
    file::InMemoryFile,
    path_storage::{PathID, PathStorage},
};
use ry_fx_hash::FxHashMap;

/// Stores basic [`codespan_reporting`] structs for reporting diagnostics.
#[derive(Debug)]
pub struct DiagnosticsEmitter {
    /// The stream in which diagnostics is reported into.
    writer: StandardStream,

    /// The config for diagnostics reporting.
    config: Config,
}

impl Default for DiagnosticsEmitter {
    fn default() -> Self {
        Self::new()
    }
}

/// Multi file diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiFileDiagnostic {
    /// ID-s of the paths of the files that the diagnostics belongs to.
    pub path_ids: Vec<PathID>,

    /// Diagnostic.
    pub diagnostic: Diagnostic<PathID>,
}

/// Global diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalDiagnostics {
    /// Diagnostics associated with concrete files.
    pub single_file_diagnostics: FxHashMap<PathID, Vec<Diagnostic<()>>>,

    /// Diagnostics associated with multiple files.
    pub multi_file_diagnostics: Vec<MultiFileDiagnostic>,

    /// Context free diagnostics.
    pub context_free_diagnostics: Vec<Diagnostic<()>>,
}

impl Default for GlobalDiagnostics {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalDiagnostics {
    /// Creates a new instance of [`GlobalDiagnostics`].
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            single_file_diagnostics: FxHashMap::default(),
            multi_file_diagnostics: Vec::new(),
            context_free_diagnostics: Vec::new(),
        }
    }

    /// Adds a diagnostic associated with a single file.
    pub fn save_single_file_diagnostic(&mut self, path_id: PathID, diagnostic: Diagnostic<()>) {
        match self.single_file_diagnostics.get_mut(&path_id) {
            Some(diagnostics_mut) => diagnostics_mut.push(diagnostic),
            None => {
                self.single_file_diagnostics
                    .insert(path_id, vec![diagnostic]);
            }
        }
    }

    /// Adds a diagnostics associated with a single file.
    pub fn save_single_file_diagnostics(
        &mut self,
        path_id: PathID,
        diagnostics: Vec<Diagnostic<()>>,
    ) {
        match self.single_file_diagnostics.get_mut(&path_id) {
            Some(diagnostics_mut) => diagnostics_mut.extend(diagnostics),
            None => {
                self.single_file_diagnostics.insert(path_id, diagnostics);
            }
        }
    }

    /// Returns `true` if diagnostics are fatal.
    #[inline]
    #[must_use]
    pub fn is_fatal(&self) -> bool {
        self.context_free_diagnostics
            .iter()
            .all(|d| !is_fatal_sevirity(d.severity))
            && self
                .single_file_diagnostics
                .iter()
                .all(|d| d.1.iter().all(|d| !is_fatal_sevirity(d.severity)))
            && self
                .multi_file_diagnostics
                .iter()
                .all(|d| !is_fatal_sevirity(d.diagnostic.severity))
    }

    /// Returns `true` if diagnostics are ok.
    #[inline]
    #[must_use]
    pub fn is_ok(&self) -> bool {
        !self.is_fatal()
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

impl Files<'_> for EmptyDiagnosticsManager {
    type FileId = ();
    type Name = EmptyName;
    type Source = EmptySource;

    fn name(&self, _: ()) -> Result<Self::Name, files::Error> {
        Ok(EmptyName)
    }

    fn source(&'_ self, _: ()) -> Result<Self::Source, files::Error> {
        Ok(EmptySource)
    }

    fn line_index(&'_ self, _: (), _: usize) -> Result<usize, files::Error> {
        panic!("line_index() is not implemented for EmptyDiagnosticsManager")
    }

    fn line_range(&'_ self, _: (), _: usize) -> Result<std::ops::Range<usize>, files::Error> {
        panic!("line_range() is not implemented for EmptyDiagnosticsManager")
    }
}

impl DiagnosticsEmitter {
    /// Emit the diagnostic not associated with a file.
    #[allow(clippy::missing_panics_doc)]
    pub fn emit_context_free_diagnostic(&self, diagnostic: &Diagnostic<()>) {
        term::emit(
            &mut self.writer.lock(),
            &self.config,
            &EmptyDiagnosticsManager,
            diagnostic,
        )
        .unwrap();
    }

    /// Emit diagnostics not associated with a particular file.
    pub fn emit_context_free_diagnostics(&self, diagnostics: &[Diagnostic<()>]) {
        for diagnostic in diagnostics {
            self.emit_context_free_diagnostic(diagnostic);
        }
    }

    /// Emit diagnostics associated with a particular file. If the file
    /// cannot be read, stops executing (no panic, diagnostic is just ignored).
    ///
    /// # Panics
    /// * If the file with a given path does not exist.
    /// * If the file path id cannot be resolved in the path storage.
    pub fn emit_particular_file_diagnostics(
        &self,
        path_storage: &PathStorage,
        file_path_id: PathID,
        file_diagnostics: &[Diagnostic<()>],
    ) {
        let path = path_storage.resolve_path_or_panic(file_path_id);
        let file = InMemoryFile::new_or_panic(&path);

        for diagnostic in file_diagnostics {
            term::emit(&mut self.writer.lock(), &self.config, &file, diagnostic).unwrap();
        }
    }

    /// Emit all of the single file diagnostics.
    pub fn emit_all_single_file_diagnostics(
        &self,
        path_storage: &PathStorage,
        diagnostics: &FxHashMap<PathID, Vec<Diagnostic<()>>>,
    ) {
        for diagnostic in diagnostics {
            self.emit_particular_file_diagnostics(path_storage, *diagnostic.0, diagnostic.1);
        }
    }

    /// Emit global diagnostics.
    pub fn emit_global_diagnostics(
        &self,
        path_storage: &PathStorage,
        global_diagnostics: &GlobalDiagnostics,
    ) {
        self.emit_context_free_diagnostics(&global_diagnostics.context_free_diagnostics);
        self.emit_all_single_file_diagnostics(
            path_storage,
            &global_diagnostics.single_file_diagnostics,
        );
    }

    /// Create a new [`DiagnosticsEmitter`] instance.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self {
            writer: StandardStream::stderr(ColorChoice::Always),
            config: Config::default(),
        }
    }

    /// Set the stream in which diagnostics is reported into.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // false-positive clippy lint
    pub fn with_diagnostics_writer(mut self, writer: StandardStream) -> Self {
        self.writer = writer;
        self
    }

    /// Set the config for diagnostics reporting.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // false-positive clippy lint
    pub fn with_diagnostics_config(mut self, config: Config) -> Self {
        self.config = config;
        self
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
pub const fn is_fatal_sevirity(severity: Severity) -> bool {
    matches!(severity, Severity::Error | Severity::Bug)
}

/// Builds a diagnostic for a **single file**!.
pub trait BuildSingleFileDiagnostic {
    /// Convert [`self`] into [`Diagnostic`].
    #[must_use]
    fn build(&self) -> Diagnostic<()>;
}

/// Builds a diagnostic for **multiple files**.
pub trait BuildMultiFileDiagnostic {
    /// Convert [`self`] into [`Diagnostic`].
    #[must_use]
    fn build(&self) -> Diagnostic<PathID>;
}
