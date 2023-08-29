use std::fs;

use stellar_diagnostics::diagnostic::Diagnostic;
use stellar_diagnostics::DiagnosticsEmitter;
use stellar_interner::PathInterner;
use stellar_manifest::parse_manifest;

pub fn command(filepath: &str) {
    let path_interner = PathInterner::new();
    let diagnostics_emitter = DiagnosticsEmitter::new(&path_interner);

    match fs::read_to_string(filepath) {
        Err(..) => {
            diagnostics_emitter.emit_context_free_diagnostic(
                &Diagnostic::error().with_message(format!("cannot read the file {filepath}")),
            );
        }
        Ok(source) => match parse_manifest(source) {
            Err(err) => {
                diagnostics_emitter.emit_context_free_diagnostic(
                    &Diagnostic::error().with_message(format!(
                        "cannot parse the manifest file due to the error: {err}"
                    )),
                );
            }
            Ok(manifest) => {
                println!("{manifest:?}");
            }
        },
    }
}
