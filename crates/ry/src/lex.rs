use crate::prefix::log_with_prefix;
use ry_interner::Interner;
use ry_lexer::Lexer;
use std::{fs, process::exit};

pub fn command(filepath: &str, show_locations: bool) {
    match fs::read_to_string(filepath) {
        Ok(source) => {
            let mut interner = Interner::default();
            let mut lexer = Lexer::new(0, &source, &mut interner);
            let mut current_token_index = 0;

            loop {
                let token = lexer.next_token();

                if token.raw.eof() {
                    break;
                } else {
                    if show_locations {
                        println!(
                            "{:08}: [{}]@{}..{}",
                            current_token_index,
                            token.raw,
                            token.span.start(),
                            token.span.end()
                        );
                    } else {
                        println!("{:08}: [{}]", current_token_index, token.raw);
                    }

                    current_token_index += 1;
                }
            }
        }
        Err(_) => {
            log_with_prefix("error", ": cannot read given file");
            exit(1);
        }
    }
}
