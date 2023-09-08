#![cfg(feature = "debug")]
use std::time::Instant;

use stellar_ast_lowering::LowerToHir;
use stellar_database::State;
use stellar_diagnostics::DiagnosticsEmitter;
use stellar_parser::parse_package_source_files;
use stellar_typechecker::resolution::{
    collect_definitions::CollectDefinitions, resolve_imports::ResolveImports,
};

use crate::log::{log_error, log_info};

pub fn command() {
    let mut state = State::new();
    let mut diagnostics_emitter = DiagnosticsEmitter::new();
    let mut now = Instant::now();

    match parse_package_source_files(&mut state, ".") {
        Err(err) => {
            log_error(err);
        }
        Ok(ast) => {
            log_info(
                format!("Parsed"),
                format!("in {}s", now.elapsed().as_secs_f64()),
            );

            now = Instant::now();

            let hir = LowerToHir::run_all(&mut state, ast);

            log_info("Lowered", format!("in {}s", now.elapsed().as_secs_f64()));

            now = Instant::now();

            CollectDefinitions::run_all(&mut state, &hir);
            ResolveImports::run_all(&mut state, &hir);

            log_info("Analyzed", format!("in {}s", now.elapsed().as_secs_f64()));

            diagnostics_emitter.emit_global_diagnostics(state.diagnostics());
        }
    };
}
