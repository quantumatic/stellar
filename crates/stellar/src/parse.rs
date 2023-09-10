#![cfg(feature = "debug")]

use std::{io::Write, time::Instant};

use stellar_database::State;
use stellar_diagnostics::DiagnosticsEmitter;
use stellar_filesystem::file_utils::make_unique_file;
use stellar_interner::{PathId, DUMMY_IDENTIFIER_ID};
use stellar_parser::read_and_parse_module;

use crate::log::{log_error, log_info};

pub fn command(filepath: &str) {
    let mut diagnostics_emitter = DiagnosticsEmitter::new();
    let mut state = State::new();
    let now = Instant::now();

    match read_and_parse_module(&mut state, DUMMY_IDENTIFIER_ID, PathId::from(filepath)) {
        Err(..) => {
            log_error(format!("cannot read the file {filepath}"));
        }
        Ok(parsed) => {
            let parsing_time = now.elapsed().as_secs_f64();
            log_info("Parsed", format!("in {parsing_time}s"));

            let diagnostics = state.into_diagnostics();

            diagnostics_emitter.emit_global_diagnostics(&diagnostics);

            if diagnostics.is_ok() {
                let now = Instant::now();
                let ast_string = serde_json::to_string(parsed.ast()).unwrap();

                log_info("Serialized", format!("in {}s", now.elapsed().as_secs_f64()));

                let (filename, file) = make_unique_file("ast", "json");
                file.expect("Cannot create `ast (n).json` file")
                    .write_all(ast_string.as_bytes())
                    .unwrap_or_else(|_| panic!("Cannot write to file {filename}"));

                log_info("Emitted", format!("AST in `{filename}`"));
            }
        }
    };
}
