//! Defines everything needed for proper error reporting.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png",
    html_favicon_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png"
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
use ry_filesystem::in_memory_file_storage::InMemoryFileStorage;
use ry_fx_hash::FxHashSet;
use ry_interner::{PathID, PathInterner};

/// Stores basic [`codespan_reporting`] structs for reporting diagnostics.
#[derive(Debug)]
pub struct DiagnosticsEmitter<'p> {
    /// The stream in which diagnostics is reported into.
    writer: StandardStream,

    /// The config for diagnostics reporting.
    config: Config,

    /// The files that are involved in the diagnostics are temporarily stored here.
    file_storage: InMemoryFileStorage<'p>,
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
    /// Files that are involved in the diagnostics.
    pub files_involved: FxHashSet<PathID>,

    /// Diagnostics associated with files.
    pub file_diagnostics: Vec<Diagnostic<PathID>>,

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
            files_involved: FxHashSet::default(),
            file_diagnostics: vec![],
            context_free_diagnostics: Vec::new(),
        }
    }

    /// Adds a diagnostic associated with a single file.
    #[inline]
    pub fn add_single_file_diagnostic(
        &mut self,
        file_path_id: PathID,
        diagnostic: Diagnostic<PathID>,
    ) {
        self.files_involved.insert(file_path_id);
        self.file_diagnostics.push(diagnostic);
    }

    /// Adds diagnostics associated with a single file.
    #[inline]
    pub fn add_single_file_diagnostics(
        &mut self,
        file_path_id: PathID,
        diagnostic: impl IntoIterator<Item = Diagnostic<PathID>>,
    ) {
        self.files_involved.insert(file_path_id);
        self.file_diagnostics.extend(diagnostic);
    }

    /// Adds a diagnostic associated with some files.
    #[inline]
    pub fn add_file_diagnostic(
        &mut self,
        files_involved: impl IntoIterator<Item = PathID>,
        diagnostic: Diagnostic<PathID>,
    ) {
        self.files_involved.extend(files_involved);
        self.file_diagnostics.push(diagnostic);
    }

    /// Adds diagnostics associated with some files.
    #[inline]
    pub fn add_file_diagnostics(
        &mut self,
        files_involved: impl IntoIterator<Item = PathID>,
        diagnostics: impl IntoIterator<Item = Diagnostic<PathID>>,
    ) {
        self.files_involved.extend(files_involved);
        self.file_diagnostics.extend(diagnostics);
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
        self.context_free_diagnostics
            .iter()
            .all(|d| !is_fatal_sevirity(d.severity))
            && self
                .file_diagnostics
                .iter()
                .all(|d| !is_fatal_sevirity(d.severity))
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

impl<'p> DiagnosticsEmitter<'p> {
    /// Create a new [`DiagnosticsEmitter`] instance.
    #[inline]
    #[must_use]
    pub fn new(path_interner: &'p PathInterner) -> Self {
        Self {
            writer: StandardStream::stderr(ColorChoice::Always),
            config: Config::default(),
            file_storage: InMemoryFileStorage::new(path_interner),
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

    /// Emit the diagnostic not associated with a file.
    #[inline]
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
    #[inline]
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
    #[inline]
    pub fn emit_file_diagnostic(&self, diagnostic: &Diagnostic<PathID>) {
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
    pub fn emit_file_diagnostics<'a>(
        &self,
        diagnostics: impl IntoIterator<Item = &'a Diagnostic<PathID>>,
    ) {
        for diagnostic in diagnostics {
            self.emit_file_diagnostic(diagnostic);
        }
    }

    /// Add files involved in the diagnostics into the file storage (if needed).
    #[allow(single_use_lifetimes)] // anonymous lifetimes in traits are unstable
    fn initialize_file_storage<'a>(
        &mut self,
        files_involved: impl IntoIterator<Item = &'a PathID>,
    ) {
        for file_path_id in files_involved {
            self.file_storage.read_and_add_file_or_panic(*file_path_id);
        }
    }

    /// Emit global diagnostics.
    #[inline]
    pub fn emit_global_diagnostics(&mut self, global_diagnostics: &GlobalDiagnostics) {
        self.initialize_file_storage(&global_diagnostics.files_involved);
        self.emit_context_free_diagnostics(&global_diagnostics.context_free_diagnostics);
        self.emit_file_diagnostics(&global_diagnostics.file_diagnostics);
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

/// Builds a diagnostic struct.
pub trait BuildDiagnostic {
    /// Convert [`self`] into [`Diagnostic`].
    #[must_use]
    fn build(&self) -> Diagnostic<PathID>;
}
