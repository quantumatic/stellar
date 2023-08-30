use std::{io::Write, time::Instant};

use parking_lot::RwLock;
use stellar_diagnostics::diagnostic::Diagnostic;
use stellar_diagnostics::{Diagnostics, DiagnosticsEmitter};
use stellar_filesystem::file_utils::make_unique_file;
use stellar_interner::PathID;
use stellar_parser::read_and_parse_module;

use crate::prefix::log_with_left_padded_prefix;

pub fn command(filepath: &str) {
    let diagnostics = RwLock::new(Diagnostics::new());
    let mut diagnostics_emitter = DiagnosticsEmitter::new();

    let now = Instant::now();

    match read_and_parse_module(PathID::from(filepath), &diagnostics) {
        Err(..) => {
            diagnostics_emitter.emit_context_free_diagnostic(
                &Diagnostic::error().with_message(format!("cannot read the file {filepath}")),
            );
        }
        Ok(ast) => {
            let parsing_time = now.elapsed().as_secs_f64();
            log_with_left_padded_prefix("Parsed", format!("in {parsing_time}s"));

            let diagnostics = diagnostics.into_inner();

            diagnostics_emitter.emit_global_diagnostics(&diagnostics);

            if diagnostics.is_ok() {
                let now = Instant::now();
                let ast_string = serde_json::to_string(&ast).unwrap();

                log_with_left_padded_prefix(
                    "Serialized",
                    format!("in {}s", now.elapsed().as_secs_f64()),
                );

                let (filename, file) = make_unique_file("ast", "json");
                file.expect("Cannot create `ast (n).json` file")
                    .write_all(ast_string.as_bytes())
                    .unwrap_or_else(|_| panic!("Cannot write to file {filename}"));

                log_with_left_padded_prefix("Emitted", format!("AST in `{filename}`"));
            }
        }
    };
}
