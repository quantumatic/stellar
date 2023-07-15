use std::{fs, path::PathBuf, process::exit};

use ry_filesystem::path_interner::PathInterner;
use ry_interner::Interner;
use ry_lexer::Lexer;

use crate::prefix::log_with_prefix;

pub fn command(filepath: &str, show_locations: bool) {
    match fs::read_to_string(filepath) {
        Ok(source) => {
            let mut interner = Interner::default();
            let mut path_interner = PathInterner::new();
            let file_path_id = path_interner.get_or_intern_path(PathBuf::from(filepath));

            let mut lexer = Lexer::new(file_path_id, &source, &mut interner);
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
                            token.location.start,
                            token.location.end,
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
