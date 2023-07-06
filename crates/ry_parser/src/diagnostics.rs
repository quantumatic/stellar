//! Defines diagnostics for parser.

use codespan_reporting::diagnostic::Diagnostic;
use ry_ast::{
    token::{LexError, Token},
    ItemKind,
};
use ry_diagnostics::{BuildDiagnostic, CompilerDiagnostic};
use ry_span::span::Span;
use std::fmt::Display;

/// Represents list of expected tokens.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Expected(pub Vec<String>);

/// Allows to construct [`Expected`] object shorter:
///
/// ```
/// use ry_parser::{expected, diagnostics::Expected};
///
/// assert_eq!(expected!("a", "b"), Expected(vec!["a".to_owned(), "b".to_owned()]));
/// ```
#[macro_export]
macro_rules! expected {
    ($($e:expr),*) => {{
        $crate::diagnostics::Expected(vec![$($e.to_string()),*])
    }};
}

/// Context in which the unnecessary visibility qualifier error is found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnnecessaryVisibilityQualifierContext {
    /// ```ry
    /// pub impl A for B {}
    /// ^^^
    /// ```
    Impl,

    /// ```ry
    /// pub trait F {
    ///     pub fun t() {}
    ///     ^^^
    ///
    ///     pub type A;
    ///     ^^^
    /// }
    /// ```
    TraitItem {
        /// Location of a trait name.
        name_span: Span,
    },
}

/// An enum which represents diagnostic encountered during parsing stage.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ParseDiagnostic {
    /// A lexing error.
    LexError(LexError),

    /// When a token is unexpected.
    UnexpectedTokenError {
        /// The token that was not expected.
        got: Token,

        /// Tokens that were expected.
        expected: Expected,

        /// AST Node at which the error occurred while parsing.
        node: String,
    },

    /// Integer overflow.
    IntegerOverflowError {
        /// Location of number when parsing which, overflow happened.
        span: Span,
    },

    /// Float overflow.
    FloatOverflowError {
        /// Location of number when parsing which, overflow happened.
        span: Span,
    },

    /// When unnecessary `pub` is found.
    UnnecessaryVisibilityQualifierError {
        /// Location of `pub`.
        span: Span,

        /// Context in which the error is found.
        context: UnnecessaryVisibilityQualifierContext,
    },

    /// When got EOF instead of close brace at the of the item.
    EOFInsteadOfCloseBrace {
        /// Type of item in which error occurred.
        item_kind: ItemKind,

        /// Location of item name.
        item_span: Span,

        /// EOF token span.
        span: Span,
    },
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

impl BuildDiagnostic for ParseDiagnostic {
    fn build(&self) -> CompilerDiagnostic {
        match self {
            Self::LexError(error) =>
                Diagnostic::error()
                    .with_message(error.raw.to_string())
                    .with_code("E000")
                    .with_labels(vec![error.span.to_primary_label()]),
            Self::UnexpectedTokenError {
                got,
                expected,
                node,
            } =>
                Diagnostic::error()
                    .with_message(format!("unexpected {}", got.raw))
                    .with_code("E001")
                    .with_labels(vec![got.span.to_primary_label()
                        .with_message(format!("expected {expected} for {node}"))]),
            Self::IntegerOverflowError { span } =>
                Diagnostic::error()
                    .with_message("unexpected integer overflow".to_owned())
                    .with_code("E002")
                    .with_labels(vec![span.to_primary_label()
                        .with_message("error appeared when parsing this integer")])
                    .with_notes(vec![
                        "note: integer cannot exceed the maximum value of `u64` (u64.max() == 18_446_744_073_709_551_615)".to_owned(),
                        "note: you can use exponent to do so, but be careful!".to_owned()
                    ]),
            Self::FloatOverflowError { span } =>
                Diagnostic::error()
                    .with_message("unexpected float overflow".to_owned())
                    .with_code("E003")
                    .with_labels(vec![span.to_primary_label()
                        .with_message("error appeared when parsing this float literal")
                    ])
                    .with_notes(vec![
                        "note: float literal cannot exceed the maximum value of `f64` (f64.max() == 1.7976931348623157E+308)".to_owned(),
                        "note: you can use exponent to do so, but be careful, especially when working with floats!".to_owned()
                    ]),
            Self::UnnecessaryVisibilityQualifierError { span, context } => {
                let mut labels = vec![span.to_primary_label().with_message("consider removing this `pub`")];

                if let UnnecessaryVisibilityQualifierContext::TraitItem { name_span } = context {
                    labels.push(name_span.to_secondary_label().with_message("happened when analyzing the trait definition"));
                }

                Diagnostic::error()
                    .with_message("unnecessary visibility qualifier".to_owned())
                    .with_code("E003")
                    .with_labels(labels)
                    .with_notes(
                        match context {
                            UnnecessaryVisibilityQualifierContext::Impl => {
                                vec!["note: using `pub` will not make the type implementation public".to_owned()]
                            }
                            UnnecessaryVisibilityQualifierContext::TraitItem { .. } => {
                                vec![
                                    "note: using `pub` for trait item will not make the item public".to_owned(),
                                    "note: all trait items are public by default".to_owned(),
                                ]
                            }
                        }
                    )
                }
            Self::EOFInsteadOfCloseBrace { item_kind, item_span, span } =>
                Diagnostic::error()
                    .with_message("unexpected end of file".to_owned())
                    .with_code("E001")
                    .with_labels(vec![
                        item_span.to_primary_label()
                            .with_message(format!("happened when parsing this {}", item_kind.to_string())),
                        span.to_secondary_label()
                            .with_message("consider adding `}`".to_owned())
                    ]),
        }
    }
}
