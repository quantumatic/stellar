use clap::{arg, Parser, Subcommand};
use codespan_reporting::files::SimpleFiles;
use ry_ast::token::RawToken::EndOfFile;
use ry_lexer::Lexer;
use ry_report::{Reporter, ReporterState};
use std::{fs, process::exit};
use string_interner::StringInterner;

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
    let reporter = ReporterState::default();

    let mut string_interner = StringInterner::new();
    let mut files = SimpleFiles::<&str, &str>::new();

    match Cli::parse().command {
        Commands::Lex {
            filepath,
            show_locations,
        } => match fs::read_to_string(filepath) {
            Ok(contents) => {
                let mut lexer = Lexer::new(&contents, &mut string_interner);
                let mut current_token_index = 0;

                loop {
                    let token = lexer.next().unwrap();

                    if token.value.is(EndOfFile) {
                        break;
                    }

                    if show_locations {
                        println!(
                            "{current_token_index}: [{}]@{}..{}",
                            token.value, token.span.start, token.span.end
                        );
                    } else {
                        println!("{current_token_index}: [{}]", token.value);
                    }

                    current_token_index += 1;
                }
            }
            Err(_) => {
                reporter.emit_global_error("cannot read given file");
                exit(1);
            }
        },
        Commands::Parse { filepath } => {
            let filepath = &filepath;

            match fs::read_to_string(filepath) {
                Ok(contents) => {
                    let file_id = files.add(&filepath, &contents);
                    let mut parser = ry_parser::Parser::new(&contents, &mut string_interner);

                    let ast = parser.parse();

                    match ast {
                        Ok(program_unit) => {
                            println!("{:?}", program_unit);
                        }
                        Err(e) => {
                            e.emit_diagnostic(&reporter, &files, file_id);

                            reporter
                                .emit_global_error("cannot output AST due to the previous errors");

                            exit(1);
                        }
                    }
                }
                Err(_) => {
                    reporter.emit_global_error("cannot read given file");
                    exit(1);
                }
            }
        }
        Commands::Serialize {
            filepath,
            resolve_docstrings,
        } => {
            let filepath = &filepath;

            match fs::read_to_string(filepath) {
                Ok(contents) => {
                    let file_id = files.add(&filepath, &contents);
                    let mut parser = ry_parser::Parser::new(&contents, &mut string_interner);

                    let ast = parser.parse();

                    match ast {
                        Ok(program_unit) => {
                            let mut serializer = ry_ast_serializer::ASTSerializer::new(
                                &string_interner,
                                resolve_docstrings,
                            );
                            println!("{}", serializer.serialize(&program_unit));
                        }
                        Err(e) => {
                            e.emit_diagnostic(&reporter, &files, file_id);

                            reporter
                                .emit_global_error("cannot output AST due to the previous errors");

                            exit(1);
                        }
                    }
                }
                Err(_) => {
                    reporter.emit_global_error("cannot read given file");
                    exit(1);
                }
            }
        }
        #[allow(unused_variables)]
        Commands::Graphviz { filepath } => todo!(),
    }
}
