use std::{io::Write, path::PathBuf, time::Instant};

use codespan_reporting::diagnostic::Diagnostic;
use ry_ast::serialize::serialize_ast;
use ry_diagnostics::{DiagnosticsEmitter, GlobalDiagnostics};
use ry_filesystem::path_storage::PathStorage;
use ry_interner::Interner;
use ry_parser::read_and_parse_module;

use crate::{prefix::log_with_left_padded_prefix, unique_file::create_unique_file};

pub fn command(path_str: &str) {
    let mut path_storage = PathStorage::new();
    let file_path_id = path_storage.add_path(PathBuf::from(path_str));

    let mut diagnostics = GlobalDiagnostics::new();
    let mut interner = Interner::default();

    let diagnostics_emitter = DiagnosticsEmitter::new();

    let now = Instant::now();

    match read_and_parse_module(&path_storage, file_path_id, &mut diagnostics, &mut interner) {
        Err(..) => {
            diagnostics_emitter.emit_context_free_diagnostic(
                &Diagnostic::error().with_message(format!("cannot read the file {}", path_str)),
            );
        }
        Ok(ast) => {
            log_with_left_padded_prefix("Parsed", path_str);
            let parsing_time = now.elapsed().as_secs_f64();

            diagnostics_emitter.emit_global_diagnostics(&path_storage, &diagnostics);

            if diagnostics.is_ok() {
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
