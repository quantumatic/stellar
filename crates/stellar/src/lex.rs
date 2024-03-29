#![cfg(feature = "debug")]
use std::{fs, process::exit};

use stellar_interner::PathId;
use stellar_lexer::Lexer;

use crate::log::log_info;

pub fn command(filepath: &str, show_locations: bool) {
    if let Ok(source) = fs::read_to_string(filepath) {
        let mut lexer = Lexer::new(PathId(1), &source);
        let mut current_token_index = 0;

        print!("0x000000: ");

        loop {
            let token = lexer.next_token();

            if token.raw.eof() {
                break;
            }

            if show_locations {
                print!(
                    "{: <25}",
                    format!(
                        "{}@{}..{}",
                        token.raw, token.location.start, token.location.end,
                    )
                );
            } else {
                print!("{: <15}", token.raw.to_string());
            }

            current_token_index += 1;

            if show_locations && current_token_index % 3 == 0
                || !show_locations && current_token_index % 5 == 0
            {
                println!();
                print!("{:#08x}: ", current_token_index);
            }
        }
    } else {
        log_info("error", ": cannot read given file");
        exit(1);
    }
}
