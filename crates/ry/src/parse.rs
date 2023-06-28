use crate::prefix::{create_unique_file, log_with_left_padded_prefix, log_with_prefix};
use ry_ast::serialize::serialize_ast;
use ry_diagnostics::{is_fatal, DiagnosticsEmitter};
use ry_interner::Interner;
use ry_parser::parse_module;
use ry_source_file::{source_file::SourceFile, workspace::Workspace};
use std::{fs, io::Write, path::Path, process::exit, time::Instant};

pub fn command(filepath: &str) {
    match fs::read_to_string(filepath) {
        Ok(source) => {
            let mut workspace = Workspace::new();
            let file = SourceFile::new(Path::new(filepath), &source);
            let file_id = workspace.add_file(&file);

            log_with_left_padded_prefix("Parsing", filepath);

            let mut diagnostics = vec![];
            let mut interner = Interner::default();

            let now = Instant::now();
            let ast = parse_module(file_id, &file, &mut diagnostics, &mut interner);
            let parsing_time = now.elapsed().as_secs_f64();

            DiagnosticsEmitter::new(&workspace).emit_diagnostics(&diagnostics);

            if !is_fatal(&diagnostics) {
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
        Err(_) => {
            log_with_prefix("error", ": cannot read given file");
            exit(1);
        }
    }
}
