#![warn(
    clippy::all,
    clippy::doc_markdown,
    clippy::dbg_macro,
    clippy::todo,
    clippy::mem_forget,
    clippy::filter_map_next,
    clippy::needless_continue,
    clippy::needless_borrow,
    clippy::match_wildcard_for_single_variants,
    clippy::mismatched_target_os,
    clippy::match_on_vec_items,
    clippy::imprecise_flops,
    clippy::suboptimal_flops,
    clippy::lossy_float_literal,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::fn_params_excessive_bools,
    clippy::inefficient_to_string,
    clippy::linkedlist,
    clippy::macro_use_imports,
    clippy::option_option,
    clippy::verbose_file_reads,
    clippy::unnested_or_patterns,
    rust_2018_idioms,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    nonstandard_style,
    unused_import_braces,
    unused_qualifications
)]
#![deny(
    clippy::await_holding_lock,
    clippy::if_let_mutex,
    clippy::indexing_slicing,
    clippy::mem_forget,
    clippy::ok_expect,
    clippy::unimplemented,
    clippy::unwrap_used,
    unsafe_code,
    unstable_features,
    unused_results
)]
#![allow(clippy::match_single_binding, clippy::inconsistent_struct_constructor)]

use crate::prefix::log_with_prefix;
use clap::{arg, Parser, Subcommand};
use new_project::create_new_project_folder;
use prefix::{create_unique_file, log_with_left_padded_prefix};
use ry_diagnostics::DiagnosticsEmitterBuilder;
use ry_interner::Interner;
use ry_lexer::Lexer;
use ry_source_file::{source_file::SourceFile, source_file_manager::SourceFileManager};
use std::{fs, io::Write, path::Path, process::exit, time::Instant};

mod new_project;
mod prefix;

#[derive(clap::Parser)]
#[command(name = "ry")]
#[command(about = "Ry programming language compiler cli", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Lex {
        filepath: String,
        #[arg(long)]
        show_locations: bool,
    },
    Parse {
        filepath: String,
    },
    New {
        project_name: String,
    },
}

fn main() {
    let mut file_manager = SourceFileManager::new();
    let mut diagnostics_emitter = DiagnosticsEmitterBuilder::new(&mut file_manager).build();

    let mut interner = Interner::default();

    match Cli::parse().command {
        Commands::Lex {
            filepath,
            show_locations,
        } => match fs::read_to_string(filepath) {
            Ok(source) => {
                let mut lexer = Lexer::new(0, &source, &mut interner);
                let mut current_token_index = 0;

                loop {
                    let token = lexer.next_token();

                    if token.unwrap().eof() {
                        break;
                    } else {
                        if show_locations {
                            println!(
                                "{:08}: [{}]@{}..{}",
                                current_token_index,
                                token.unwrap(),
                                token.span().start(),
                                token.span().end()
                            );
                        } else {
                            println!("{:08}: [{}]", current_token_index, token.unwrap());
                        }

                        current_token_index += 1;
                    }
                }
            }
            Err(_) => {
                log_with_prefix("error", ": cannot read given file");
                exit(1);
            }
        },
        Commands::Parse { filepath } => {
            let filepath = &filepath;

            match fs::read_to_string(filepath) {
                Ok(source) => {
                    let file_id =
                        diagnostics_emitter.add_file(SourceFile::new(Path::new(filepath), &source));

                    let mut diagnostics = vec![];
                    let mut cursor =
                        ry_parser::Cursor::new(file_id, &source, &mut interner, &mut diagnostics);

                    let now = Instant::now();

                    log_with_left_padded_prefix("Parsing", filepath);

                    let ast = cursor.parse();

                    diagnostics_emitter.emit_diagnostics(diagnostics.as_slice());

                    let ast_string = format!("{:?}", ast);

                    let (filename, mut file) = create_unique_file("ast", "txt");
                    file.write_all(ast_string.as_bytes())
                        .unwrap_or_else(|_| panic!("Cannot write to file {}", filename));

                    log_with_left_padded_prefix(
                        "Parsed",
                        format!("in {}s", now.elapsed().as_secs_f64()),
                    );
                    log_with_left_padded_prefix("Emitted", format!("AST in `{}`", filename));
                }
                Err(_) => {
                    log_with_prefix("error", ": cannot read given file");
                    exit(1);
                }
            }
        }
        Commands::New { project_name } => {
            create_new_project_folder(&project_name);
        }
    }
}
