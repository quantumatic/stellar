use std::{io::Write, path::PathBuf, time::Instant};

use codespan_reporting::diagnostic::Diagnostic;
use ry_ast_lowering::LoweringContext;
use ry_diagnostics::{Diagnostics, DiagnosticsEmitter};
use ry_interner::{IdentifierInterner, PathInterner};
use ry_parser::read_and_parse_module;

use crate::{prefix::log_with_left_padded_prefix, unique_file::create_unique_file};

pub fn command(path_str: &str) {
    let mut path_interner = PathInterner::new();
    let mut identifier_interner = IdentifierInterner::new();

    let file_path_id = path_interner.get_or_intern(PathBuf::from(path_str));

    let mut diagnostics = Diagnostics::new();
    let mut diagnostics_emitter = DiagnosticsEmitter::new(&path_interner);

    let mut now = Instant::now();

    match read_and_parse_module(
        &path_interner,
        file_path_id,
        &mut diagnostics,
        &mut identifier_interner,
    ) {
        Err(..) => {
            diagnostics_emitter.emit_context_free_diagnostic(
                &Diagnostic::error().with_message(format!("cannot read the file {path_str}")),
            );
        }
        Ok(ast) => {
            log_with_left_padded_prefix("Parsed", format!("in {}s", now.elapsed().as_secs_f64()));

            now = Instant::now();

            let mut lowering_context = LoweringContext::new(file_path_id, &mut diagnostics);
            let hir = lowering_context.lower(ast);

            log_with_left_padded_prefix("Lowered", format!("in {}s", now.elapsed().as_secs_f64()));

            diagnostics_emitter.emit_global_diagnostics(&diagnostics);

            if diagnostics.is_ok() {
                now = Instant::now();

                let hir_string = serde_json::to_string(&hir).unwrap();

                log_with_left_padded_prefix(
                    "Serialized",
                    format!("in {}s", now.elapsed().as_secs_f64()),
                );

                let (filename, mut file) = create_unique_file("hir", "json");
                file.write_all(hir_string.as_bytes())
                    .unwrap_or_else(|_| panic!("Cannot write to file {filename}"));

                log_with_left_padded_prefix("Emitted", format!("AST in `{filename}`"));
            }
        }
    };
}
