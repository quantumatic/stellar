// #![doc(
//     html_logo_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png",
//     html_favicon_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png"
// )]
// #![cfg_attr(not(test), forbid(clippy::unwrap_used))]
// #![warn(missing_docs, clippy::dbg_macro)]
// #![deny(
//     // rustc lint groups https://doc.rust-lang.org/rustc/lints/groups.html
//     warnings,
//     future_incompatible,
//     let_underscore,
//     nonstandard_style,
//     rust_2018_compatibility,
//     rust_2018_idioms,
//     rust_2021_compatibility,
//     unused,
//     // rustc allowed-by-default lints https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html
//     macro_use_extern_crate,
//     meta_variable_misuse,
//     missing_abi,
//     missing_copy_implementations,
//     missing_debug_implementations,
//     non_ascii_idents,
//     noop_method_call,
//     single_use_lifetimes,
//     trivial_casts,
//     trivial_numeric_casts,
//     unreachable_pub,
//     unsafe_op_in_unsafe_fn,
//     unused_crate_dependencies,
//     unused_import_braces,
//     unused_lifetimes,
//     unused_qualifications,
//     unused_tuple_struct_fields,
//     variant_size_differences,
//     // rustdoc lints https://doc.rust-lang.org/rustdoc/lints.html
//     rustdoc::broken_intra_doc_links,
//     rustdoc::private_intra_doc_links,
//     rustdoc::missing_crate_level_docs,
//     rustdoc::private_doc_tests,
//     rustdoc::invalid_codeblock_attributes,
//     rustdoc::invalid_rust_codeblocks,
//     rustdoc::bare_urls,
//     // clippy categories https://doc.rust-lang.org/clippy/
//     clippy::all,
//     clippy::correctness,
//     clippy::suspicious,
//     clippy::style,
//     clippy::complexity,
//     clippy::perf,
//     clippy::pedantic,
//     clippy::nursery,
// )]
// #![allow(
//     clippy::module_name_repetitions,
//     clippy::too_many_lines,
//     clippy::option_if_let_else
// )]

use codespan_reporting::{
    diagnostic::Diagnostic,
    files::SimpleFiles,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
        Config,
    },
};

/// Stores basic `codespan_reporting` structs for reporting errors.
pub struct ReporterState<'f> {
    pub writer: StandardStream,
    pub config: Config,
    pub files: SimpleFiles<&'f str, &'f str>,
}

impl<'f> ReporterState<'f> {
    pub fn emit_global_error(&self, msg: &str) {
        term::emit(
            &mut self.writer.lock(),
            &self.config,
            &self.files,
            &Diagnostic::error().with_message(msg),
        )
        .expect("emit_global_diagnostic() failed");
    }

    pub fn add_file(&mut self, filename: &'f str, source: &'f str) -> usize {
        self.files.add(filename, source)
    }
}

impl Default for ReporterState<'_> {
    fn default() -> Self {
        Self {
            writer: StandardStream::stderr(ColorChoice::Always),
            config: codespan_reporting::term::Config::default(),
            files: SimpleFiles::new(),
        }
    }
}

pub trait Reporter<'source> {
    fn emit_diagnostic(&self, reporter: &ReporterState) {
        term::emit(
            &mut reporter.writer.lock(),
            &reporter.config,
            &reporter.files,
            &self.build_diagnostic(),
        )
        .expect("emit_diagnostic() failed")
    }

    fn build_diagnostic(&self) -> Diagnostic<usize>;
}
