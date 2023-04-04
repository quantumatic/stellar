//! Error and result implementation for the parser.

use codespan_reporting::diagnostic::{Diagnostic, Label};
use ry_ast::{span::*, token::*};
use ry_report::Reporter;
use std::fmt::Display;

/// An enum which represents errors encountered during parsing stage.
#[derive(Debug)]
pub enum ParseError {
    /// A lexing error.
    Lexer {
        /// The error that occured during lexing.
        error: Spanned<LexError>,
    },

    /// When a token is unexpected.
    UnexpectedToken {
        /// The token that was not expected.
        got: Token,

        /// The description of what was expected.
        expected: String,

        /// AST Node at which the error occurred while parsing.
        node: String,
    },
}

impl From<Spanned<LexError>> for ParseError {
    fn from(error: Spanned<LexError>) -> Self {
        Self::Lexer { error }
    }
}

/// Result of a parsing.
pub type ParseResult<T> = Result<T, ParseError>;

impl ParseError {
    pub(crate) fn lexer(error: Spanned<LexError>) -> Self {
        error.into()
    }

    pub(crate) fn unexpected_token<E, N>(got: Token, expected: E, node: N) -> Self
    where
        E: Into<String>,
        N: Into<String>,
    {
        match got.unwrap() {
            RawToken::Error(lexer_error) => Self::lexer((*lexer_error).with_span(got.span())),
            _ => Self::UnexpectedToken {
                got,
                expected: expected.into(),
                node: node.into(),
            },
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lexer { error } => error.unwrap().fmt(f),
            Self::UnexpectedToken {
                got,
                expected,
                node,
            } => {
                f.write_fmt(format_args!(
                    "expected {expected} for {node}, got {}",
                    got.unwrap()
                ))?;
                Ok(())
            }
        }
    }
}

impl Reporter<'_> for ParseError {
    fn build_diagnostic(&self, file_id: usize) -> Diagnostic<usize> {
        match self {
            Self::Lexer { error } => Diagnostic::error()
                .with_message(self.to_string())
                .with_code("E000")
                .with_labels(vec![
                    Label::primary(file_id, error.span()).with_message("error appeared here")
                ]),
            Self::UnexpectedToken {
                got,
                expected,
                node,
            } => Diagnostic::error()
                .with_message(format!("unexpected {}", got.unwrap()))
                .with_code("E001")
                .with_labels(vec![Label::primary(file_id, got.span())
                    .with_message(format!("expected {expected} for {node}"))]),
        }
    }
}
