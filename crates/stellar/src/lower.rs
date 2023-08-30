use std::io::Write;
use std::time::Instant;

use parking_lot::RwLock;
use stellar_ast_lowering::LowerExt;
use stellar_diagnostics::diagnostic::Diagnostic;
use stellar_diagnostics::{Diagnostics, DiagnosticsEmitter};
use stellar_filesystem::file_utils::make_unique_file;
use stellar_interner::PathID;
use stellar_parser::read_and_parse_module;

use crate::prefix::log_with_left_padded_prefix;

pub fn command(filepath: &str) {
    let diagnostics = RwLock::new(Diagnostics::new());
    let mut diagnostics_emitter = DiagnosticsEmitter::new();

    let mut now = Instant::now();

    let path_id = PathID::from(filepath);

    match read_and_parse_module(path_id, &diagnostics) {
        Err(..) => {
            diagnostics_emitter.emit_context_free_diagnostic(
                &Diagnostic::error().with_message(format!("cannot read the file {filepath}")),
            );
        }
        Ok(ast) => {
            log_with_left_padded_prefix("Parsed", format!("in {}s", now.elapsed().as_secs_f64()));

            now = Instant::now();

            let hir = ast.lower(&diagnostics);

            log_with_left_padded_prefix("Lowered", format!("in {}s", now.elapsed().as_secs_f64()));

            let diagnostics = diagnostics.into_inner();

            diagnostics_emitter.emit_global_diagnostics(&diagnostics);

            if diagnostics.is_ok() {
                now = Instant::now();

                let hir_string = serde_json::to_string(&hir).unwrap();

                log_with_left_padded_prefix(
                    "Serialized",
                    format!("in {}s", now.elapsed().as_secs_f64()),
                );

                let (filename, file) = make_unique_file("hir", "json");
                file.expect("Cannot create `hir (n).json` file")
                    .write_all(hir_string.as_bytes())
                    .unwrap_or_else(|_| panic!("Cannot write to file {filename}"));

                log_with_left_padded_prefix("Emitted", format!("HIR in `{filename}`"));
            }
        }
    };
}
