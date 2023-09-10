#![cfg(feature = "debug")]

use std::io::Write;
use std::time::Instant;

use stellar_ast_lowering::LowerToHir;
use stellar_database::State;
use stellar_diagnostics::DiagnosticsEmitter;
use stellar_filesystem::file_utils::make_unique_file;
use stellar_interner::{PathId, DUMMY_IDENTIFIER_ID};
use stellar_parser::read_and_parse_module;

use crate::log::{log_error, log_info};

pub fn command(filepath: &str) {
    let mut diagnostics_emitter = DiagnosticsEmitter::new();
    let mut state = State::new();
    let filepath = PathId::from(filepath);

    let mut now = Instant::now();

    match read_and_parse_module(&mut state, DUMMY_IDENTIFIER_ID, filepath) {
        Err(..) => {
            log_error(format!("cannot read the file {filepath}"));
        }
        Ok(ast) => {
            log_info("Parsed", format!("in {}s", now.elapsed().as_secs_f64()));

            now = Instant::now();

            let hir = LowerToHir::run_all(&mut state, vec![ast]);
            let hir = hir.first_key_value().unwrap().1;

            log_info("Lowered", format!("in {}s", now.elapsed().as_secs_f64()));

            diagnostics_emitter.emit_global_diagnostics(state.diagnostics());

            if state.diagnostics().is_ok() {
                now = Instant::now();

                let hir_string = serde_json::to_string(hir).unwrap();

                log_info("Serialized", format!("in {}s", now.elapsed().as_secs_f64()));

                let (filename, file) = make_unique_file("hir", "json");
                file.expect("Cannot create `hir (n).json` file")
                    .write_all(hir_string.as_bytes())
                    .unwrap_or_else(|_| panic!("Cannot write to file {filename}"));

                log_info("Emitted", format!("HIR in `{filename}`"));
            }
        }
    };
}
