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
mod collect_definitions;
mod collect_signatures;
mod lex;
mod log;
mod lower;
mod parse;
mod parse_manifest;
mod resolve_imports;
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
    #[cfg(feature = "debug")]
    #[command(about = "Debug mode: tokenize a given source file")]
    Lex {
        filepath: String,
        #[arg(long)]
        show_locations: bool,
    },
    #[cfg(feature = "debug")]
    #[command(about = "Debug mode: tokenize a given source file")]
    Tokenize {
        filepath: String,
        #[arg(long)]
        show_locations: bool,
    },
    #[cfg(feature = "debug")]
    #[command(about = "Debug mode: tokenize a given source file")]
    Scan {
        filepath: String,
        #[arg(long)]
        show_locations: bool,
    },
    #[cfg(feature = "debug")]
    #[command(about = "Debug mode: parse a given source file and serialize its AST")]
    Ast { filepath: String },
    #[cfg(feature = "debug")]
    #[command(about = "Debug mode: parse a given source file and serialize its AST")]
    Parse { filepath: String },
    #[cfg(feature = "debug")]
    #[command(about = "Debug mode: parse a given source file, lower its AST and serialize HIR")]
    Hir { filepath: String },
    #[cfg(feature = "debug")]
    #[command(about = "Debug mode: parse a given source file, lower its AST and serialize HIR")]
    LowerAst { filepath: String },
    #[cfg(feature = "debug")]
    #[command(about = "Debug mode: parses a given manifest file")]
    ParseManifest { filepath: String },
    #[command(about = "Creates a new package")]
    New { package_name: String },
    #[command(about = "Prints current version of the compiler")]
    CompilerVersion,
    #[command(about = "Prints current version of the standart library")]
    StdVersion,
    #[cfg(feature = "debug")]
    #[command(about = "Debug mode: debug collect definitions")]
    CollectDefinitions,
    #[cfg(feature = "debug")]
    #[command(
        about = "Debug mode: debug collect definitions, resolve imports and collect signatures"
    )]
    CollectSignatures,
    #[cfg(feature = "debug")]
    #[command(about = "Debug mode: debug collect definitions and resolve imports")]
    ResolveImports,
    #[command(about = "Prints current version of the package manager (Stellar repository)")]
    PackageManagerVersion,
}

fn main() {
    #[cfg(feature = "debug")]
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(std::env::var("STELLAR_LOG").unwrap_or_else(|_| "off".to_owned()))
        .without_time()
        .with_ansi(false)
        .with_target(false)
        .with_level(false)
        .init();

    match Cli::parse().command {
        #[cfg(feature = "debug")]
        Commands::CollectDefinitions => collect_definitions::command(),
        #[cfg(feature = "debug")]
        Commands::CollectSignatures => collect_signatures::command(),
        #[cfg(feature = "debug")]
        Commands::ResolveImports => resolve_imports::command(),
        Commands::CompilerVersion => version::compiler_version_command(),
        Commands::StdVersion => version::std_version_command(),
        Commands::PackageManagerVersion => version::package_manager_version_command(),
        #[cfg(feature = "debug")]
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
        #[cfg(feature = "debug")]
        Commands::Ast { filepath } | Commands::Parse { filepath } => {
            parse::command(&filepath);
        }
        #[cfg(feature = "debug")]
        Commands::Hir { filepath } | Commands::LowerAst { filepath } => {
            lower::command(&filepath);
        }
        #[cfg(feature = "debug")]
        Commands::ParseManifest { filepath } => {
            parse_manifest::command(&filepath);
        }
        Commands::New { package_name: _ } => {
            todo!()
        }
    }
}
