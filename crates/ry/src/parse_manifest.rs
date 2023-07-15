use std::fs;

use codespan_reporting::diagnostic::Diagnostic;
use ry_diagnostics::DiagnosticsEmitter;
use ry_filesystem::path_interner::PathInterner;
use ry_manifest::parse_manifest;

pub fn command(filepath: &str) {
    let path_interner = PathInterner::new();
    let diagnostics_emitter = DiagnosticsEmitter::new(&path_interner);

    match fs::read_to_string(filepath) {
        Err(..) => {
            diagnostics_emitter.emit_context_free_diagnostic(
                &Diagnostic::error().with_message(format!("cannot read the file {}", filepath)),
            );
        }
        Ok(source) => match parse_manifest(source) {
            Err(err) => {
                diagnostics_emitter.emit_context_free_diagnostic(&Diagnostic::error().with_message(
                    format!("cannot parse the manifest file due to the error: {err}"),
                ))
            }
            Ok(manifest) => {
                println!("{:?}", manifest);
            }
        },
    }
}
