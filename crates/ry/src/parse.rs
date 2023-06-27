use crate::prefix::{create_unique_file, log_with_left_padded_prefix, log_with_prefix};
use ry_ast::serialize::serialize_ast_or_panic;
use ry_diagnostics::DiagnosticsEmitter;
use ry_parser::{parse_module, ParseState};
use ry_source_file::{source_file::SourceFile, source_file_manager::SourceFileManager};
use std::{fs, io::Write, path::Path, process::exit, time::Instant};

pub fn command(filepath: &str) {
    match fs::read_to_string(filepath) {
        Ok(source) => {
            let mut file_manager = SourceFileManager::new();
            let source_file = SourceFile::new(Path::new(filepath), &source);
            let file_id = file_manager.add_file(&source_file);
            let diagnostics_emitter = DiagnosticsEmitter::new(&file_manager);

            let now = Instant::now();

            log_with_left_padded_prefix("Parsing", filepath);

            let mut state = ParseState::new(file_id, &source_file);
            let (ast, diagnostics, interner) = parse_module(&mut state);

            log_with_left_padded_prefix("Parsed", format!("in {}s", now.elapsed().as_secs_f64()));

            diagnostics_emitter.emit_diagnostics(diagnostics.as_slice());

            let ast_string = serialize_ast_or_panic(&ast, interner, &file_manager);

            let (filename, mut file) = create_unique_file("ast", "txt");
            file.write_all(ast_string.as_bytes())
                .unwrap_or_else(|_| panic!("Cannot write to file {}", filename));

            log_with_left_padded_prefix("Emitted", format!("AST in `{}`", filename));
        }
        Err(_) => {
            log_with_prefix("error", ": cannot read given file");
            exit(1);
        }
    }
}
