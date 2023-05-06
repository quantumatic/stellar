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
use prefix::create_unique_file;
use ry_interner::Interner;
use ry_lexer::Lexer;
use ry_parser::ParserState;
use ry_report::{Reporter, ReporterState};
use std::{fs, io::Write, process::exit, time::Instant};

mod error;
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
}

fn main() {
    let mut reporter = ReporterState::default();

    let mut interner = Interner::default();

    match Cli::parse().command {
        Commands::Lex {
            filepath,
            show_locations,
        } => match fs::read_to_string(filepath) {
            Ok(contents) => {
                let mut lexer = Lexer::new(0, &contents, &mut interner);
                let mut current_token_index = 0;

                loop {
                    let token = lexer.next();

                    if let Some(token) = token {
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
                    } else {
                        break;
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
                Ok(contents) => {
                    let file_id = reporter.add_file(filepath, &contents);
                    let mut parser = ParserState::new(file_id, &contents, &mut interner);

                    let now = Instant::now();

                    log_with_prefix("   Parsing ", filepath);

                    let ast = parser.parse();

                    match ast {
                        Ok(program_unit) => {
                            let json = mytry!(serde_json::to_string_pretty(&program_unit));

                            let (filename, mut file) = create_unique_file("ast", "json");
                            mytry!(file.write_all(json.as_bytes()));

                            log_with_prefix(
                                "    Parsed ",
                                format!("in {}s", now.elapsed().as_secs_f64()),
                            );
                            log_with_prefix("   Emitted ", format!("AST in `{}`", filename));
                        }
                        Err(e) => {
                            e.emit_diagnostic(&reporter);

                            log_with_prefix(
                                "error",
                                ": cannot emit AST due to the previous errors",
                            );

                            exit(1);
                        }
                    }
                }
                Err(_) => {
                    log_with_prefix("error", ": cannot read given file");
                    exit(1);
                }
            }
        }
    }
}
