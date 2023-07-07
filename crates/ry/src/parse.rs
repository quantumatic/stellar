use crate::{prefix::log_with_left_padded_prefix, unique_file::create_unique_file};
use codespan_reporting::diagnostic::Diagnostic;
use ry_ast::serialize::serialize_ast;
use ry_diagnostics::{check_file_diagnostics, DiagnosticsEmitter, DiagnosticsStatus};
use ry_interner::Interner;
use ry_parser::parse_module;
use std::{io::Write, path::Path, time::Instant};

pub fn command(path_str: &str) {
    let path = Path::new(path_str);

    let diagnostics_emitter = DiagnosticsEmitter::new();
    let mut diagnostics = vec![];

    let mut interner = Interner::default();

    let now = Instant::now();

    match parse_module(path, &mut diagnostics, &mut interner) {
        Err(..) => {
            diagnostics_emitter.emit_context_free_diagnostic(
                &Diagnostic::error().with_message(format!("cannot read the file {}", path_str)),
            );
        }
        Ok(ast) => {
            log_with_left_padded_prefix("Parsed", path_str);
            let parsing_time = now.elapsed().as_secs_f64();

            diagnostics_emitter.emit_file_diagnostics(path, &diagnostics);

            if check_file_diagnostics(&diagnostics) == DiagnosticsStatus::Ok {
                log_with_left_padded_prefix("Parsed", format!("in {}s", parsing_time));

                let now = Instant::now();
                let ast_string = serialize_ast(&ast, &interner);

                log_with_left_padded_prefix(
                    "Serialized",
                    format!("in {}s", now.elapsed().as_secs_f64()),
                );

                let (filename, mut file) = create_unique_file("ast", "txt");
                file.write_all(ast_string.as_bytes())
                    .unwrap_or_else(|_| panic!("Cannot write to file {}", filename));

                log_with_left_padded_prefix("Emitted", format!("AST in `{}`", filename));
            }
        }
    };
}
