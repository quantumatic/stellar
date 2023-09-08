#![cfg(feature = "debug")]
use std::{fs, process::exit};

use stellar_interner::PathID;
use stellar_lexer::Lexer;

use crate::log::log_info;

pub fn command(filepath: &str, show_locations: bool) {
    if let Ok(source) = fs::read_to_string(filepath) {
        let mut lexer = Lexer::new(PathID(1), &source);
        let mut current_token_index = 0;

        loop {
            let token = lexer.next_token();

            if token.raw.eof() {
                break;
            }

            if show_locations {
                println!(
                    "{:08}: [{}]@{}..{}",
                    current_token_index, token.raw, token.location.start, token.location.end,
                );
            } else {
                println!("{:08}: [{}]", current_token_index, token.raw);
            }

            current_token_index += 1;
        }
    } else {
        log_info("error", ": cannot read given file");
        exit(1);
    }
}
