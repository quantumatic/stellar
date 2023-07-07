//! Defines everything needed for proper error reporting.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png",
    html_favicon_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
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
use std::{collections::HashMap, fmt::Display, path::Path};

use codespan_reporting::{
    diagnostic::{Diagnostic, Severity},
    files::{self, Files},
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
        Config,
    },
};
use ry_filesystem::file::InMemoryFile;

/// Stores basic `codespan_reporting` structs for reporting diagnostics.
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

/// Diagnostics type alias.
pub type RyDiagnostic = Diagnostic<()>;

/// Global diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalDiagnostics<'a> {
    /// Diagnostics associated with concrete files.
    pub file_diagnostics: HashMap<&'a Path, Vec<RyDiagnostic>>,

    /// Context free diagnostics.
    pub context_free_diagnostics: Vec<RyDiagnostic>,
}

impl Default for GlobalDiagnostics<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> GlobalDiagnostics<'a> {
    /// Creates a new instance of [`GlobalDiagnostics`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            file_diagnostics: HashMap::new(),
            context_free_diagnostics: Vec::new(),
        }
    }

    /// Adds a diagnostic to a file.
    pub fn add_file_diagnostic(&mut self, path: &'a Path, diagnostic: RyDiagnostic) {
        match self.file_diagnostics.get_mut(path) {
            Some(diagnostics_mut) => diagnostics_mut.push(diagnostic),
            None => {
                self.file_diagnostics.insert(path, vec![diagnostic]);
            }
        }
    }

    /// Adds diagnostics to a file.
    pub fn add_file_diagnostics(&mut self, path: &'a Path, diagnostics: Vec<RyDiagnostic>) {
        match self.file_diagnostics.get_mut(path) {
            Some(diagnostics_mut) => diagnostics_mut.extend(diagnostics),
            None => {
                self.file_diagnostics.insert(path, diagnostics);
            }
        }
    }
}

/// Empty diagnostics manager (implements [`Files`]).
///
/// [`Files`]: codespan_reporting::files::Files
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
    pub fn emit_context_free_diagnostic(&self, diagnostic: &RyDiagnostic) {
        term::emit(
            &mut self.writer.lock(),
            &self.config,
            &EmptyDiagnosticsManager,
            diagnostic,
        )
        .expect("emit_global_diagnostic() failed");
    }

    /// Emit diagnostics not associated with a particular file.
    pub fn emit_context_free_diagnostics(&self, diagnostics: &[RyDiagnostic]) {
        for diagnostic in diagnostics {
            self.emit_context_free_diagnostic(diagnostic);
        }
    }

    /// Emit diagnostics associated with a particular file.
    pub fn emit_file_diagnostics(&self, path: &Path, file_diagnostics: &[RyDiagnostic]) {
        let file = InMemoryFile::new_or_panic(path);

        for diagnostic in file_diagnostics {
            term::emit(&mut self.writer.lock(), &self.config, &file, diagnostic)
                .expect("Cannot emit the diagnostic");
        }
    }

    /// Emit a list of diagnostic.
    pub fn emit_global_diagnostics(&self, diagnostics: &HashMap<&Path, Vec<RyDiagnostic>>) {
        for diagnostic in diagnostics {
            self.emit_file_diagnostics(diagnostic.0, diagnostic.1);
        }
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

/// Check if diagnostics associated with a particular file are fatal.
///
/// Note: ID is not required.
#[must_use]
pub fn check_file_diagnostics(file_diagnostics: &[RyDiagnostic]) -> DiagnosticsStatus {
    for diagnostic in file_diagnostics {
        if matches!(diagnostic.severity, Severity::Error | Severity::Bug) {
            return DiagnosticsStatus::Fatal;
        }
    }

    DiagnosticsStatus::Ok
}

/// Check if diagnostics are fatal.
#[must_use]
pub fn check_global_diagnostics(diagnostics: &GlobalDiagnostics<'_>) -> DiagnosticsStatus {
    for diagnostic in &diagnostics.file_diagnostics {
        if check_file_diagnostics(diagnostic.1) == DiagnosticsStatus::Fatal {
            return DiagnosticsStatus::Fatal;
        }
    }

    DiagnosticsStatus::Ok
}

/// Anything that can be reported using [`DiagnosticsEmitter`].
pub trait BuildDiagnostic {
    /// Convert [`self`] into [`CompilerDiagnostic`].
    fn build(&self) -> RyDiagnostic;
}
