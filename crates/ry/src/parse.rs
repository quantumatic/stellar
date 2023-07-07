use crate::{prefix::log_with_left_padded_prefix, unique_file::create_unique_file};
use ry_ast::serialize::serialize_ast;
use ry_diagnostics::{check_file_diagnostics, DiagnosticsEmitter, DiagnosticsStatus};
use ry_filesystem::path_resolver::FilePathResolver;
use ry_interner::Interner;
use ry_parser::{parse_module, InitializeParseStateError};
use std::{io::Write, path::Path, time::Instant};

pub fn command(filepath: &str) {
    let mut path_resolver = FilePathResolver::new();
    let file_id = path_resolver.add_path(Path::new(filepath));

    let diagnostics_emitter = DiagnosticsEmitter::new(&path_resolver);
    let mut diagnostics = vec![];

    let mut interner = Interner::default();

    let now = Instant::now();

    match parse_module(file_id, &path_resolver, &mut diagnostics, &mut interner) {
        Err(InitializeParseStateError::ReadFileError(..)) => {
            diagnostics_emitter.emit_global_error(format!("cannot read the file {}", filepath));
        }
        Err(InitializeParseStateError::ResolvePathError(..)) => unreachable!(),
        Ok(ast) => {
            log_with_left_padded_prefix("Parsed", filepath);
            let parsing_time = now.elapsed().as_secs_f64();

            diagnostics_emitter.emit_file_diagnostics(file_id, &diagnostics);

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
