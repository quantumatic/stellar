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
    #[command(arg_required_else_help = true)]
    Lex {
        #[arg(required = true)]
        filepath: String,
        show_locations: bool,
    },
    #[command(arg_required_else_help = true)]
    Parse {
        #[arg(required = true)]
        filepath: String,
    },
    #[command(arg_required_else_help = true)]
    Serialize {
        #[arg(required = true)]
        filepath: String,
    },
    #[command(arg_required_else_help = true)]
    Graphviz {
        #[arg(required = true)]
        filepath: String,
    },
}

fn main() {
    let reporter = ReporterState::default();

    let mut identifier_interner = StringInterner::new();
    let mut files = SimpleFiles::<&str, &str>::new();

    match Cli::parse().command {
        Commands::Lex {
            filepath,
            show_locations,
        } => match fs::read_to_string(filepath) {
            Ok(contents) => {
                let mut lexer = Lexer::new(&contents, &mut identifier_interner);
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
                    let mut identifier_interner = StringInterner::default();
                    let mut parser = ry_parser::Parser::new(&contents, &mut identifier_interner);

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
        Commands::Serialize { filepath } => {
            let filepath = &filepath;

            match fs::read_to_string(filepath) {
                Ok(contents) => {
                    let file_id = files.add(&filepath, &contents);
                    let mut parser = ry_parser::Parser::new(&contents, &mut identifier_interner);

                    let ast = parser.parse();

                    match ast {
                        Ok(program_unit) => {
                            let mut serializer =
                                ry_ast_serializer::ASTSerializer::new(&identifier_interner);
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
