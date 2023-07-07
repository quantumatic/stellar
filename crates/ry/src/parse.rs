use crate::{prefix::log_with_left_padded_prefix, unique_file::create_unique_file};
use ry_ast::serialize::serialize_ast;
use ry_diagnostics::{check_file_diagnostics, DiagnosticsEmitter, DiagnosticsStatus};
use ry_filesystem::path_resolver::PathResolver;
use ry_interner::Interner;
use ry_parser::parse_module_or_panic;
use std::{io::Write, path::Path, time::Instant};

pub fn command(filepath: &str) {
    let mut path_resolver = PathResolver::new();
    let file_id = path_resolver.add_path(Path::new(filepath));

    log_with_left_padded_prefix("Parsing", filepath);

    let mut diagnostics = vec![];
    let mut interner = Interner::default();

    let now = Instant::now();
    let ast = parse_module_or_panic(file_id, &path_resolver, &mut diagnostics, &mut interner);
    let parsing_time = now.elapsed().as_secs_f64();

    DiagnosticsEmitter::new(&path_resolver).emit_file_diagnostics(file_id, &diagnostics);

    if check_file_diagnostics(&diagnostics) == DiagnosticsStatus::Ok {
        log_with_left_padded_prefix("Parsed", format!("in {}s", parsing_time));

        let now = Instant::now();
        let ast_string = serialize_ast(&ast, &interner);

        log_with_left_padded_prefix("Serialized", format!("in {}s", now.elapsed().as_secs_f64()));

        let (filename, mut file) = create_unique_file("ast", "txt");
        file.write_all(ast_string.as_bytes())
            .unwrap_or_else(|_| panic!("Cannot write to file {}", filename));

        log_with_left_padded_prefix("Emitted", format!("AST in `{}`", filename));
    }
}
