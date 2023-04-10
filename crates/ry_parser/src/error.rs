//! Error and result implementation for the state.
use codespan_reporting::diagnostic::{Diagnostic, Label};
use ry_ast::{
    span::{At, Span, Spanned},
    token::{LexError, RawToken, Token},
};
use ry_report::Reporter;
use std::fmt::Display;

/// Represents list of expected tokens.
#[derive(Debug)]
pub struct Expected(Vec<String>);

macro_rules! expected {
    ($($e:expr),*) => {{
        vec![$($e.to_string()),*]
    }};
}

pub(crate) use expected;

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

        /// Tokens that were expected.
        expected: Expected,

        /// AST Node at which the error occurred while parsing.
        node: String,
    },

    /// Integer overflow.
    IntegerOverflow {
        /// Location of number when parsing which, overflow happened.
        at: Span,
    },

    /// Float overflow.
    FloatOverflow {
        /// Location of number when parsing which, overflow happened.
        at: Span,
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

    pub(crate) fn unexpected_token<N>(got: Token, expected: Vec<String>, node: N) -> Self
    where
        N: Into<String>,
    {
        match got.inner {
            RawToken::Error(lexer_error) => Self::lexer(lexer_error.at(got.span)),
            _ => Self::UnexpectedToken {
                got,
                expected: Expected(expected),
                node: node.into(),
            },
        }
    }

    pub(crate) fn integer_overflow(at: Span) -> Self {
        Self::IntegerOverflow { at }
    }

    pub(crate) fn float_overflow(at: Span) -> Self {
        Self::FloatOverflow { at }
    }
}

impl Display for Expected {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let len = self.0.len() - 1;

        f.write_fmt(format_args!(
            "{}",
            self.0
                .iter()
                .enumerate()
                .map(|(idx, token)| {
                    format!(
                        "{}{token}",
                        if idx == 0 {
                            ""
                        } else if idx == len {
                            " or "
                        } else {
                            ", "
                        }
                    )
                })
                .collect::<String>()
        ))
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lexer { error } => error.inner.fmt(f),
            Self::UnexpectedToken {
                got,
                expected,
                node,
            } => {
                f.write_fmt(format_args!(
                    "expected {expected} for {node}, got {}",
                    got.inner
                ))?;
                Ok(())
            }
            Self::IntegerOverflow { .. } => f.write_str("integer overflow"),
            Self::FloatOverflow { .. } => f.write_str("float overflow"),
        }
    }
}

impl Reporter<'_> for ParseError {
    fn build_diagnostic(&self, file_id: usize) -> Diagnostic<usize> {
        match self {
            Self::Lexer { error } => Diagnostic::error()
                .with_message(self.to_string())
                .with_code("E000")
                .with_labels(vec![Label::primary(file_id, error.span)]),
            Self::UnexpectedToken {
                got,
                expected,
                node,
            } => Diagnostic::error()
                .with_message(format!("unexpected {}", got.inner))
                .with_code("E001")
                .with_labels(vec![Label::primary(file_id, got.span)
                    .with_message(format!("expected {expected} for {node}"))]),
            Self::IntegerOverflow { at } => Diagnostic::error()
                .with_message(format!("unexpected integer overflow"))
                .with_labels(vec![Label::primary(file_id, *at)
                    .with_message("error appeared when parsing this integer")])
                .with_notes(vec![
                    "note: integer cannot exceed maximum value of u64 (u64.max() == 18_446_744_073_709_551_615)".to_owned(),
                    "note: you can use exponent to do so, but be carefull!".to_owned()
                ]),
            Self::FloatOverflow { at } => Diagnostic::error()
                .with_message(format!("unexpected number overflow"))
                .with_labels(vec![Label::primary(file_id, *at)
                    .with_message("error appeared when parsing this float literal")])
                    .with_notes(vec![
                        "note: float literal cannot exceed maximum value of f64 (f64.max() == 1.7976931348623157E+308)".to_owned(),
                        "note: you can use exponent to do so, but be carefull, especially when working with floats!".to_owned()
                    ]),
        }
    }
}
