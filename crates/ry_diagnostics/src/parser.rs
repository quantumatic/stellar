//! Error and result implementation for the cursor.
use crate::Report;
use codespan_reporting::diagnostic::Diagnostic;
use ry_ast::{
    token::{LexError, Token},
    ItemKind,
};
use ry_source_file::span::{Span, Spanned};
use std::fmt::Display;

/// Represents list of expected tokens.
#[derive(Debug, PartialEq, Eq)]
pub struct Expected(pub Vec<String>);

/// Allows to construct [`Expected`] object shorter:
///
/// ```
/// use ry_diagnostics::{expected, parser::Expected};
///
/// assert_eq!(expected!("a", "b"), Expected(vec!["a".to_owned(), "b".to_owned()]));
/// ```
#[macro_export]
macro_rules! expected {
    ($($e:expr),*) => {{
        $crate::parser::Expected(vec![$($e.to_string()),*])
    }};
}

/// An enum which represents diagnostic encountered during parsing stage.
#[derive(Debug)]
pub enum ParseDiagnostic {
    /// A lexing error.
    LexError(Spanned<LexError>),

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
        at: Span,
    },

    /// Float overflow.
    FloatOverflowError {
        /// Location of number when parsing which, overflow happened.
        at: Span,
    },

    /// Error that suggests adding `;` after expression in statements block.
    NoSemicolonAfterExpressionError {
        /// Location of expression which does not have corresponding `;`.
        expression_span: Span,

        /// Possible span of `;` in the future.
        at: Span,
    },

    /// Error that suggests adding `;` after any statement in statements block.
    NoSemicolonAfterStatementError {
        /// Location of the statement.
        statement_span: Span,

        /// Possible span of `;` in the future.
        at: Span,
    },

    /// When got EOF instead of close brace at the end of the statements block.
    EOFInsteadOfCloseBraceForStatementsBlockError {
        /// Location of `{`.
        statements_block_start_span: Span,

        /// EOF token span.
        at: Span,
    },

    /// When got two semicolons in a row: `;;` or semicolon immediately after `{`
    /// in the statements block.
    EmptyStatementWarning {
        /// The span of the semicolon.
        at: Span,
    },

    /// When got EOF instead of close brace at the of the item.
    EOFInsteadOfCloseBraceForItemError {
        /// Type of item in which error occurred.
        item_kind: ItemKind,

        /// Location of item name.
        item_name_span: Span,

        /// EOF token span.
        at: Span,
    },

    /// When got two `..` in struct pattern.
    MoreThanTwoRestPatternsInStructPatternMembersError {
        /// Location of struct name.
        struct_name_span: Span,

        /// Previous `..` struct member pattern location.
        previous_rest_pattern_span: Span,

        /// Current `..` struct member pattern location.
        current_rest_pattern_span: Span,
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

impl Report for ParseDiagnostic {
    fn build(&self) -> Diagnostic<usize> {
        match self {
            Self::LexError(error) =>
                Diagnostic::error()
                    .with_message(error.unwrap().to_string())
                    .with_code("E000")
                    .with_labels(vec![error.span().to_primary_label()]),
            Self::UnexpectedTokenError {
                got,
                expected,
                node,
            } =>
                Diagnostic::error()
                    .with_message(format!("unexpected {}", got.unwrap()))
                    .with_code("E001")
                    .with_labels(vec![got.span().to_primary_label()
                        .with_message(format!("expected {expected} for {node}"))]),
            Self::IntegerOverflowError { at } =>
                Diagnostic::error()
                    .with_message("unexpected integer overflow".to_owned())
                    .with_labels(vec![at.to_primary_label()
                        .with_message("error appeared when parsing this integer")])
                    .with_notes(vec![
                        "note: integer cannot exceed the maximum value of `u64` (u64.max() == 18_446_744_073_709_551_615)".to_owned(),
                        "note: you can use exponent to do so, but be careful!".to_owned()
                    ]),
            Self::FloatOverflowError { at } =>
                Diagnostic::error()
                    .with_message("unexpected float overflow".to_owned())
                    .with_labels(vec![at.to_primary_label()
                        .with_message("error appeared when parsing this float literal")])
                        .with_notes(vec![
                            "note: float literal cannot exceed the maximum value of `f64` (f64.max() == 1.7976931348623157E+308)".to_owned(),
                            "note: you can use exponent to do so, but be careful, especially when working with floats!".to_owned()
                        ]),
            Self::NoSemicolonAfterExpressionError { expression_span, at } =>
                Diagnostic::error()
                    .with_message("it seems that you forgot to put `;` after the expression")
                    .with_labels(vec![
                        at.to_secondary_label()
                            .with_message("add `;` here"),
                        expression_span.to_primary_label()
                            .with_message("happened when parsing this expression")
                    ]),
            Self::NoSemicolonAfterStatementError { statement_span, at } =>
                Diagnostic::error()
                    .with_message("it seems that you forgot to put `;` after the statement")
                    .with_labels(vec![
                        at.to_secondary_label()
                            .with_message("add `;` here"),
                        statement_span.to_primary_label()
                            .with_message("happened when parsing this statement")
                    ]),
            Self::EOFInsteadOfCloseBraceForStatementsBlockError { statements_block_start_span, at } =>
                Diagnostic::error()
                    .with_message("unexpected end of file".to_owned())
                    .with_labels(vec![
                        statements_block_start_span.to_primary_label()
                            .with_message("happened when parsing this statements block"),
                        at.to_secondary_label()
                            .with_message("consider adding `}`".to_owned())
                    ]),
            Self::EmptyStatementWarning { at } =>
                Diagnostic::warning()
                    .with_message("found empty statement".to_owned())
                    .with_labels(vec![
                        at.to_primary_label()
                            .with_message("consider removing this `;`".to_owned())
                    ])
                    .with_notes(vec![
                        "note: empty statements do not have syntactic meaning.".to_owned()
                    ]),
            Self::EOFInsteadOfCloseBraceForItemError { item_kind, item_name_span, at } =>
                Diagnostic::error()
                    .with_message("unexpected end of file".to_owned())
                    .with_labels(vec![
                        item_name_span.to_primary_label()
                            .with_message(format!("happened when parsing this {}", item_kind.to_string())),
                        at.to_secondary_label()
                            .with_message("consider adding `}`".to_owned())
                    ]),
            Self::MoreThanTwoRestPatternsInStructPatternMembersError { struct_name_span, previous_rest_pattern_span, current_rest_pattern_span } =>
                Diagnostic::error()
                    .with_message("multiple appearances of `..` in struct pattern found".to_owned())
                    .with_labels(vec![
                        struct_name_span.to_primary_label()
                            .with_message("happened when processing this struct pattern"),
                        previous_rest_pattern_span.to_secondary_label()
                            .with_message("first appearance of `..`"),
                        current_rest_pattern_span.to_secondary_label()
                            .with_message("second appearance of `..`")
                    ])
        }
    }
}
