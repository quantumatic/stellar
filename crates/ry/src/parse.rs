use std::{io::Write, path::PathBuf, time::Instant};

use parking_lot::RwLock;
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

    let now = Instant::now();

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
