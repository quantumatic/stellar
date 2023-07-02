//! # Source Files Managment.
//!
//! This crate provides utilities for working with Ry source files.
//!
//! ## [`Workspace`] and [`SourceFile`]
//!
//! - [`Workspace`] is a helper struct for working with Ry source files and also provides implementation for
//! [`Files`] in [`codespan_reporting`] for proper error reporting. It is important to make
//! sure that you added your source file into the [`Workspace`], because it would not
//! report diagnostics properly with ID being out of bonds.
//!
//! - [`SourceFile`] is an interface for working with Ry source files.
//!
//! ## [`Span`]
//!
//! [`Span`] is an interface for working with source code locations.
//!
//! [`Workspace`]: crate::workspace::Workspace
//! [`SourceFile`]: crate::file::SourceFile
//! [`Span`]: crate::span::Span
//! [`Files`]: codespan_reporting::files::Files

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

pub mod file;
pub mod path_resolver;
pub mod span;
pub mod workspace;
