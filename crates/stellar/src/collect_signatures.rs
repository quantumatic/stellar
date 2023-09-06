#![cfg(feature = "debug")]
use std::time::Instant;

use stellar_ast_lowering::LowerToHir;
use stellar_database::State;
use stellar_diagnostics::{diagnostic::Diagnostic, DiagnosticsEmitter};
use stellar_parser::parse_package_source_files;
use stellar_typechecker::{
    resolution::{collect_definitions::CollectDefinitions, resolve_imports::ResolveImports},
    signature_analysis::collect_signatures::CollectSignatures,
};

use crate::log::log;

pub fn command() {
    let mut state = State::new();
    let mut diagnostics_emitter = DiagnosticsEmitter::new();
    let mut now = Instant::now();

    match parse_package_source_files(&mut state, ".") {
        Err(err) => {
            diagnostics_emitter
                .emit_context_free_diagnostic(&Diagnostic::error().with_message(err));
        }
        Ok(ast) => {
            log(
                format!("Parsed"),
                format!("in {}s", now.elapsed().as_secs_f64()),
            );

            now = Instant::now();

            let hir = LowerToHir::run_all(&mut state, ast);

            log("Lowered", format!("in {}s", now.elapsed().as_secs_f64()));

            now = Instant::now();

            CollectDefinitions::run_all(&mut state, &hir);
            ResolveImports::run_all(&mut state, &hir);
            CollectSignatures::run_all(&mut state, &hir);

            log("Analyzed", format!("in {}s", now.elapsed().as_secs_f64()));

            diagnostics_emitter.emit_global_diagnostics(state.diagnostics());
        }
    };
}
