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
use std::{collections::HashMap, fmt::Display};

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
    path_resolver::{FileID, PathResolver},
};

/// Stores basic `codespan_reporting` structs for reporting diagnostics.
#[derive(Debug)]
pub struct DiagnosticsEmitter<'path_resolver> {
    /// The stream in which diagnostics is reported into.
    writer: StandardStream,

    /// The config for diagnostics reporting.
    config: Config,

    /// The path resolver.
    path_resolver: &'path_resolver PathResolver<'path_resolver>,
}

/// Diagnostics associated with a file.
pub type FileDiagnostic = Diagnostic<()>;

/// Global diagnostics.
pub type GlobalDiagnostics = HashMap<FileID, Vec<FileDiagnostic>>;

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

impl<'path_resolver> DiagnosticsEmitter<'path_resolver> {
    /// Emit the error not related to a conrete file.
    pub fn emit_global_error<S>(&self, message: S)
    where
        S: AsRef<str>,
    {
        term::emit(
            &mut self.writer.lock(),
            &self.config,
            &EmptyDiagnosticsManager,
            &Diagnostic::error().with_message(message.as_ref()),
        )
        .expect("emit_global_diagnostic() failed");
    }

    /// Emit diagnostics associated with a particular file.
    pub fn emit_file_diagnostics(&self, file_id: FileID, file_diagnostics: &[FileDiagnostic]) {
        let path = self
            .path_resolver
            .resolve_path(file_id)
            .expect("Cannot resolve the path needed to emit the diagnostic");
        let file = InMemoryFile::new(path).expect("Cannot open file needed to emit the diagnostic");

        for diagnostic in file_diagnostics {
            term::emit(&mut self.writer.lock(), &self.config, &file, diagnostic)
                .expect("Cannot emit the diagnostic");
        }
    }

    /// Emit a list of diagnostic.
    pub fn emit_global_diagnostics(&self, diagnostics: &HashMap<FileID, Vec<FileDiagnostic>>) {
        for diagnostic in diagnostics {
            self.emit_file_diagnostics(*diagnostic.0, diagnostic.1);
        }
    }

    /// Create a new [`DiagnosticsEmitter`] instance.
    #[must_use]
    #[inline]
    pub fn new(path_resolver: &'path_resolver PathResolver<'_>) -> Self {
        Self {
            writer: StandardStream::stderr(ColorChoice::Always),
            config: Config::default(),
            path_resolver,
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
pub fn check_file_diagnostics(file_diagnostics: &[FileDiagnostic]) -> DiagnosticsStatus {
    for diagnostic in file_diagnostics {
        if matches!(diagnostic.severity, Severity::Error | Severity::Bug) {
            return DiagnosticsStatus::Fatal;
        }
    }

    DiagnosticsStatus::Ok
}

/// Check if diagnostics are fatal.
#[must_use]
pub fn check_global_diagnostics(diagnostics: &GlobalDiagnostics) -> DiagnosticsStatus {
    for diagnostic in diagnostics {
        if check_file_diagnostics(diagnostic.1) == DiagnosticsStatus::Fatal {
            return DiagnosticsStatus::Fatal;
        }
    }

    DiagnosticsStatus::Ok
}

/// Anything that can be reported using [`DiagnosticsEmitter`].
pub trait BuildDiagnostic {
    /// Convert [`self`] into [`CompilerDiagnostic`].
    fn build(&self) -> FileDiagnostic;
}
