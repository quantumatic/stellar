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
    diagnostic::Diagnostic,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
        Config,
    },
};
use ry_source_file::{source_file::SourceFile, source_file_manager::SourceFileManager};

pub mod parser;

/// Stores basic `codespan_reporting` structs for reporting diagnostics.
#[derive(Debug)]
pub struct DiagnosticsEmitter<'a> {
    /// The stream in which diagnostics is reported into.
    pub writer: StandardStream,

    /// The config for diagnostics reporting.
    pub config: Config,

    /// The source file manager.
    pub file_manager: &'a mut SourceFileManager<'a>,
}

impl<'a> DiagnosticsEmitter<'a> {
    /// Emit the error not related to a conrete file.
    pub fn emit_global_error(&self, msg: &str) {
        term::emit(
            &mut self.writer.lock(),
            &self.config,
            self.file_manager,
            &Diagnostic::error().with_message(msg),
        )
        .expect("emit_global_diagnostic() failed");
    }

    /// Emit a diagnostic.
    pub fn emit_diagnostic(&self, diagnostic: &Diagnostic<usize>) {
        term::emit(
            &mut self.writer.lock(),
            &self.config,
            self.file_manager,
            diagnostic,
        )
        .expect("emit_diagnostic() failed");
    }

    /// Emit a list of diagnostic.
    pub fn emit_diagnostics(&self, diagnostics: &[Diagnostic<usize>]) {
        for diagnostic in diagnostics {
            self.emit_diagnostic(diagnostic);
        }
    }

    /// Add a file to the source file manager.
    pub fn add_file(&mut self, file: SourceFile<'a>) -> usize {
        self.file_manager.add_file(file)
    }
}

/// Builder for [`Reporter`].
///
/// See [`ReporterBuilder::with_diagnostics_writer`] and [`ReporterBuilder::with_diagnostics_config`]
/// for more details.
#[derive(Debug)]
pub struct DiagnosticsEmitterBuilder<'a> {
    /// The stream in which diagnostics is reported into.
    diagnostics_writer: Option<StandardStream>,

    /// The config for diagnostics reporting.
    diagnostics_config: Option<Config>,

    /// The source file manager.
    file_manager: &'a mut SourceFileManager<'a>,
}

impl<'a> DiagnosticsEmitterBuilder<'a> {
    /// Create a new [`ReporterBuilder`].
    #[must_use]
    #[inline]
    pub fn new(file_manager: &'a mut SourceFileManager<'a>) -> Self {
        Self {
            diagnostics_writer: None,
            diagnostics_config: None,
            file_manager,
        }
    }

    /// Set the stream in which diagnostics is reported into.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // clippy issue
    pub fn with_diagnostics_writer(mut self, writer: StandardStream) -> Self {
        self.diagnostics_writer = Some(writer);
        self
    }

    /// Set the config for diagnostics reporting.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // clippy issue
    pub fn with_diagnostics_config(mut self, config: Config) -> Self {
        self.diagnostics_config = Some(config);
        self
    }

    /// Build new [`Reporter`] object.
    pub fn build(self) -> DiagnosticsEmitter<'a> {
        DiagnosticsEmitter {
            writer: self
                .diagnostics_writer
                .unwrap_or_else(|| StandardStream::stderr(ColorChoice::Always)),
            config: self.diagnostics_config.unwrap_or_default(),
            file_manager: self.file_manager,
        }
    }
}

/// Anything that can be reported using [`Reporter`].
pub trait Report {
    /// Convert [`self`] into [`Diagnostic<usize>`].
    fn build(&self) -> Diagnostic<usize>;
}
