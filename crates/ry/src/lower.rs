use std::io::Write;
use std::{path::PathBuf, time::Instant};

use parking_lot::RwLock;
use ry_ast_lowering::LowerExt;
use ry_diagnostics::diagnostic::Diagnostic;
use ry_diagnostics::{Diagnostics, DiagnosticsEmitter};
use ry_filesystem::file_utils::make_unique_file;
use ry_interner::{IdentifierInterner, PathInterner};
use ry_parser::read_and_parse_module;

use crate::prefix::log_with_left_padded_prefix;

pub fn command(path_str: &str) {
    let mut path_interner = PathInterner::new();
    let identifier_interner = RwLock::new(IdentifierInterner::new());

    let file_path_id = path_interner.get_or_intern(PathBuf::from(path_str));

    let diagnostics = RwLock::new(Diagnostics::new());
    let mut diagnostics_emitter = DiagnosticsEmitter::new(&path_interner);

    let mut now = Instant::now();

    match read_and_parse_module(
        &path_interner,
        file_path_id,
        &diagnostics,
        &identifier_interner,
    ) {
        Err(..) => {
            diagnostics_emitter.emit_context_free_diagnostic(
                &Diagnostic::error().with_message(format!("cannot read the file {path_str}")),
            );
        }
        Ok(ast) => {
            log_with_left_padded_prefix("Parsed", format!("in {}s", now.elapsed().as_secs_f64()));

            now = Instant::now();

            let hir = ast.lower(file_path_id, &diagnostics);

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
