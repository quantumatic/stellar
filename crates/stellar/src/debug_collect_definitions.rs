use std::{sync::Arc, time::Instant};

use stellar_ast_lowering::LowerToHir;
use stellar_database::State;
use stellar_diagnostics::{diagnostic::Diagnostic, DiagnosticsEmitter};
use stellar_interner::PathID;
use stellar_parser::read_and_parse_module;
use stellar_typechecker::collect_definitions::CollectDefinitions;

use crate::prefix::log_with_left_padded_prefix;

pub fn command(filepath: &str) {
    let mut diagnostics_emitter = DiagnosticsEmitter::new();
    let state = Arc::new(State::new());
    let path_id = PathID::from(filepath);

    let mut now = Instant::now();

    match read_and_parse_module(path_id, state.diagnostics()) {
        Err(..) => {
            diagnostics_emitter.emit_context_free_diagnostic(
                &Diagnostic::error().with_message(format!("cannot read the file {filepath}")),
            );
        }
        Ok(ast) => {
            log_with_left_padded_prefix("Parsed", format!("in {}s", now.elapsed().as_secs_f64()));

            now = Instant::now();

            let hir = &LowerToHir::run_all(state.clone(), vec![ast]);

            log_with_left_padded_prefix("Lowered", format!("in {}s", now.elapsed().as_secs_f64()));

            now = Instant::now();

            CollectDefinitions::run_all(state.clone(), hir);

            log_with_left_padded_prefix("Analyzed", format!("in {}s", now.elapsed().as_secs_f64()));

            let diagnostics = state.diagnostics_lock();

            diagnostics_emitter.emit_global_diagnostics(&diagnostics);
        }
    };
}
