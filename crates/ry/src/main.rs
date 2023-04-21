use crate::prefix::log_with_prefix;
use clap::{arg, Parser, Subcommand};
use prefix::create_unique_file;
use ry_interner::Interner;
use ry_lexer::Lexer;
use ry_parser::ParserState;
use ry_report::{Reporter, ReporterState};
use std::{fs, io::Write, process::exit};

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
    Serialize {
        filepath: String,
        #[arg(long)]
        resolve_docstrings: bool,
    },
    Graphviz {
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
                let mut lexer = Lexer::new(&contents, &mut interner);
                let mut current_token_index = 0;

                loop {
                    let token = lexer.next();

                    if token.is_none() {
                        break;
                    }

                    let token = token.unwrap();

                    if show_locations {
                        println!(
                            "{:08}: [{}]@{}..{}",
                            current_token_index, token.inner, token.span.start, token.span.end
                        );
                    } else {
                        println!("{:08}: [{}]", current_token_index, token.inner);
                    }

                    current_token_index += 1;
                }
            }
            Err(_) => {
                log_with_prefix("error: ", "cannot read given file");
                exit(1);
            }
        },
        Commands::Parse { filepath } => {
            let filepath = &filepath;

            match fs::read_to_string(filepath) {
                Ok(contents) => {
                    let file_id = reporter.add_file(&filepath, &contents);
                    let mut parser = ParserState::new(&contents, &mut interner);

                    log_with_prefix("parsing ", filepath);

                    let ast = parser.parse();

                    match ast {
                        Ok(program_unit) => {
                            let json = serde_json::to_string_pretty(&program_unit).unwrap();
                            log_with_prefix("finished ", filepath);

                            let (filename, mut file) = create_unique_file("ast", "json");
                            file.write_all(json.to_string().as_bytes());

                            log_with_prefix("note: ", &format!("AST is written in {}", filename));
                        }
                        Err(e) => {
                            e.emit_diagnostic(&reporter, file_id);

                            reporter
                                .emit_global_error("cannot output AST due to the previous errors");

                            exit(1);
                        }
                    }
                }
                Err(_) => {
                    log_with_prefix("error: ", "cannot read given file");
                    exit(1);
                }
            }
        }
        #[allow(unused_variables)]
        Commands::Graphviz { filepath } => todo!(),
        _ => todo!(),
    }
}
