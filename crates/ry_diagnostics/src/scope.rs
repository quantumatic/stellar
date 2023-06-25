//! Defines diagnostics related to scopes.

use codespan_reporting::diagnostic::Diagnostic;
use ry_source_file::span::Span;

use crate::{BuildDiagnostic, CompilerDiagnostic};

/// Diagnostics related to scopes.
#[allow(missing_copy_implementations)]
#[derive(Debug, Clone)]
pub enum ScopeDiagnostic {
    /// Symbol wasnot found in the current scope.
    SymbolNotFound {
        /// The symbol itself.
        symbol: String,

        /// The place where the symbol was tried to be used.
        span: Span,
    },

    /// When trying to overwrite an existing symbol.
    SymbolOverwritten {
        /// The symbol itself.
        symbol: String,

        /// The place where the symbol was defined before.
        first_definition_span: Span,

        /// The place where the symbol was defined now.
        second_definition_span: Span,
    },
}

impl BuildDiagnostic for ScopeDiagnostic {
    fn build(&self) -> CompilerDiagnostic {
        match self {
            Self::SymbolNotFound { symbol, span } => Diagnostic::error()
                .with_message(format!("symbol `{symbol}` is not found in this scope"))
                .with_code("E005")
                .with_labels(vec![span.to_primary_label()]),
            Self::SymbolOverwritten {
                symbol,
                first_definition_span,
                second_definition_span,
            } => Diagnostic::error()
                .with_message(format!("symbol `{symbol}` cannot be overwritten"))
                .with_code("E006")
                .with_labels(vec![
                    first_definition_span
                        .to_primary_label()
                        .with_message("previously defined here"),
                    second_definition_span
                        .to_primary_label()
                        .with_message("redefined here"),
                ])
                .with_notes(vec!["note: try to give it a different name".to_owned()]),
        }
    }
}
