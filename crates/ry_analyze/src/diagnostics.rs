//! Defines diagnostics related to scopes.

use codespan_reporting::diagnostic::Diagnostic;
use ry_diagnostics::{BuildDiagnostic, CompilerDiagnostic};
use ry_span::span::Span;

/// Diagnostics related to scopes.
#[allow(missing_copy_implementations)]
#[derive(Debug, Clone)]
pub enum ScopeDiagnostic {
    /// Symbol wasnot found in the current scope.
    NotFound {
        /// The symbol itself.
        symbol: String,

        /// The place where the symbol was tried to be used.
        span: Span,
    },
}

impl BuildDiagnostic for ScopeDiagnostic {
    fn build(&self) -> CompilerDiagnostic {
        match self {
            Self::NotFound { symbol, span } => Diagnostic::error()
                .with_message(format!("`{symbol}` is not found in this scope"))
                .with_code("E004")
                .with_labels(vec![span.to_primary_label()]),
        }
    }
}
