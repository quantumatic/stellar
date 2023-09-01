#![doc(
    html_logo_url = "https://raw.githubusercontent.com/quantumatic/stellar/main/additional/icon/stellar.png",
    html_favicon_url = "https://raw.githubusercontent.com/quantumatic/stellar/main/additional/icon/stellar.png"
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
    clippy::unnested_or_patterns,
    clippy::explicit_deref_methods,
    clippy::significant_drop_tightening
)]

use clap::{Parser, Subcommand};

#[cfg(feature = "debug")]
mod debug_collect_definitions;
mod lex;
mod lower;
mod parse;
mod parse_manifest;
mod prefix;
mod version;

#[derive(Parser)]
#[command(name = "stellar")]
#[command(about = "Stellar programming language compiler cli", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Tokenizes a given source file")]
    Lex {
        filepath: String,
        #[arg(long)]
        show_locations: bool,
    },
    #[command(about = "Tokenizes a given source file")]
    Tokenize {
        filepath: String,
        #[arg(long)]
        show_locations: bool,
    },
    #[command(about = "Tokenizes a given source file")]
    Scan {
        filepath: String,
        #[arg(long)]
        show_locations: bool,
    },
    #[command(about = "Parses a given source file and serializes its AST")]
    Ast { filepath: String },
    #[command(about = "Parses a given source file and serializes its AST")]
    Parse { filepath: String },
    #[command(about = "Parses a given source file, lower its AST and serializes HIR")]
    Hir { filepath: String },
    #[command(about = "Parses a given source file, lower its AST and serializes HIR")]
    LowerAst { filepath: String },
    #[command(about = "Parses a given manifest file")]
    ParseManifest { filepath: String },
    #[command(about = "Creates a new package")]
    New { package_name: String },
    #[command(about = "Prints current version of the compiler")]
    CompilerVersion,
    #[command(about = "Prints current version of the standart library")]
    StdVersion,
    #[cfg(feature = "debug")]
    #[command(about = "Debug mode: debug collect definitions")]
    DebugCollectDefinitions,
    #[command(about = "Prints current version of the package manager (Stellar repository)")]
    PackageManagerVersion,
}

fn main() {
    #[cfg(feature = "debug")]
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(std::env::var("STELLAR_LOG").unwrap_or_else(|_| "off".to_owned()))
        .without_time()
        .with_level(false)
        .init();

    match Cli::parse().command {
        #[cfg(feature = "debug")]
        Commands::DebugCollectDefinitions => debug_collect_definitions::command(),
        Commands::CompilerVersion => version::compiler_version_command(),
        Commands::StdVersion => version::std_version_command(),
        Commands::PackageManagerVersion => version::package_manager_version_command(),
        Commands::Lex {
            filepath,
            show_locations,
        }
        | Commands::Tokenize {
            filepath,
            show_locations,
        }
        | Commands::Scan {
            filepath,
            show_locations,
        } => lex::command(&filepath, show_locations),
        Commands::Ast { filepath } | Commands::Parse { filepath } => {
            parse::command(&filepath);
        }
        Commands::Hir { filepath } | Commands::LowerAst { filepath } => {
            lower::command(&filepath);
        }
        Commands::ParseManifest { filepath } => {
            parse_manifest::command(&filepath);
        }
        Commands::New { package_name: _ } => {
            todo!()
        }
    }
}
