use codespan_reporting::diagnostic::{Diagnostic, Label};
use ry_ast::{span::*, token::*};
use ry_report::Reporter;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParserError {
    /// Error appeared in lexing stage.
    #[error("scanning error appeared in the process")]
    ErrorToken(WithSpan<LexerError>),

    /// Unexpected token [`Token`] in AST Node called
    /// [`Option<String>`], expected [`String`].
    #[error("unexpected token `{0:?}`, expected `{1}` for `{2}`")]
    UnexpectedToken(Token, String, String),
}

impl<'source> Reporter<'source> for ParserError {
    fn build_diagnostic(&self, file_id: usize) -> Diagnostic<usize> {
        match self {
            Self::ErrorToken(token) => Diagnostic::error()
                .with_message("scanning error occured")
                .with_code("E000")
                .with_labels(vec![
                    Label::primary(file_id, token.span()).with_message(token.unwrap().to_string())
                ]),
            Self::UnexpectedToken(got, expected, node_name) => {
                let mut label_message = format!("expected {expected}");

                label_message.push_str(format!(" for {node_name}").as_str());

                Diagnostic::error()
                    .with_message(format!("unexpected {}", got.unwrap()))
                    .with_code("E001")
                    .with_labels(vec![
                        Label::primary(file_id, got.span()).with_message(label_message)
                    ])
            }
        }
    }
}
