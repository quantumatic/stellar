#![doc(
    html_logo_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png",
    html_favicon_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png"
)]
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
    unsafe_op_in_unsafe_fn,
    //unused_crate_dependencies,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_tuple_struct_fields,
    variant_size_differences,
    // rustdoc lints https://doc.rust-lang.org/rustdoc/lints.html
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    //rustdoc::missing_crate_level_docs,
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
    clippy::unnested_or_patterns
)]

use std::env;

use clap::{Parser, Subcommand};

mod lex;
mod lower;
mod new;
mod parse;
mod parse_manifest;
mod prefix;
mod unique_file;

#[derive(Parser)]
#[command(name = "ry")]
#[command(about = "Ry programming language compiler cli", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Tokenize Ry source file")]
    Lex {
        filepath: String,
        #[arg(long)]
        show_locations: bool,
    },
    #[command(about = "Parse Ry source file and get its AST")]
    Ast { filepath: String },
    #[command(about = "Parse Ry source file, lower its AST and return output HIR")]
    Hir { filepath: String },
    #[command(about = "Parse Ry manifest file")]
    ParseManifest { filepath: String },
    #[command(about = "Create a new Ry package")]
    New { package_name: String },
}

fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(env::var("RY_LOG").unwrap_or_else(|_| "off".to_owned()))
        .without_time()
        .with_ansi(false)
        .init();

    match Cli::parse().command {
        Commands::Lex {
            filepath,
            show_locations,
        } => lex::command(&filepath, show_locations),
        Commands::Ast { filepath } => {
            parse::command(&filepath);
        }
        Commands::Hir { filepath } => {
            lower::command(&filepath);
        }
        Commands::ParseManifest { filepath } => {
            parse_manifest::command(&filepath);
        }
        Commands::New { package_name } => {
            new::command(&package_name);
        }
    }
}
