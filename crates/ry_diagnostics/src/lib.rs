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

use codespan_reporting::{
    diagnostic::{Diagnostic, Severity},
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
        Config,
    },
};
use ry_workspace::workspace::{FileID, Workspace};

pub mod parser;
pub mod scope;

/// Stores basic `codespan_reporting` structs for reporting diagnostics.
#[derive(Debug)]
pub struct DiagnosticsEmitter<'workspace> {
    /// The stream in which diagnostics is reported into.
    pub writer: StandardStream,

    /// The config for diagnostics reporting.
    pub config: Config,

    /// The workspace in which diagnostics are reported.
    pub workspace: &'workspace Workspace<'workspace>,
}

/// A diagnostic.
pub type CompilerDiagnostic = Diagnostic<FileID>;

impl<'workspace> DiagnosticsEmitter<'workspace> {
    /// Emit the error not related to a conrete file.
    pub fn emit_global_error(&self, msg: &str) {
        term::emit(
            &mut self.writer.lock(),
            &self.config,
            self.workspace,
            &Diagnostic::error().with_message(msg),
        )
        .expect("emit_global_diagnostic() failed");
    }

    /// Emit a diagnostic.
    pub fn emit_diagnostic(&self, diagnostic: &CompilerDiagnostic) {
        term::emit(
            &mut self.writer.lock(),
            &self.config,
            self.workspace,
            diagnostic,
        )
        .expect("emit_diagnostic() failed");
    }

    /// Emit a list of diagnostic.
    pub fn emit_diagnostics(&self, diagnostics: &Vec<CompilerDiagnostic>) {
        for diagnostic in diagnostics {
            self.emit_diagnostic(diagnostic);
        }
    }

    /// Create a new [`DiagnosticsEmitter`] instance.
    #[must_use]
    #[inline]
    pub fn new(workspace: &'workspace Workspace<'workspace>) -> Self {
        Self {
            writer: StandardStream::stderr(ColorChoice::Always),
            config: Config::default(),
            workspace,
        }
    }

    /// Set the stream in which diagnostics is reported into.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // clippy issue
    pub fn with_diagnostics_writer(mut self, writer: StandardStream) -> Self {
        self.writer = writer;
        self
    }

    /// Set the config for diagnostics reporting.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // clippy issue
    pub fn with_diagnostics_config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }
}

/// Check if diagnostics are fatal.
#[must_use]
pub fn is_fatal(diagnostics: &Vec<CompilerDiagnostic>) -> bool {
    for diagnostic in diagnostics {
        if matches!(diagnostic.severity, Severity::Error | Severity::Bug) {
            return true;
        }
    }

    false
}

/// Anything that can be reported using [`DiagnosticsEmitter`].
pub trait BuildDiagnostic {
    /// Convert [`self`] into [`CompilerDiagnostic`].
    fn build(&self) -> CompilerDiagnostic;
}
