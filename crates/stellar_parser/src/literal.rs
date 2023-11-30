use stellar_ast::{token::RawToken, Literal};

use crate::{
    diagnostics::{FloatOverflow, IntegerOverflow},
    Parse, ParseState,
};

pub(crate) struct LiteralParser;

impl Parse for LiteralParser {
    type Output = Option<Literal>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        match state.next_token.raw {
            RawToken::IntegerLiteral => {
                state.advance();

                if let Ok(value) = state
                    .resolve_current_token_str()
                    .replace('_', "")
                    .parse::<u64>()
                {
                    Some(Literal::Integer {
                        value,
                        location: state.current_token.location,
                    })
                } else {
                    state
                        .diagnostics
                        .add_diagnostic(IntegerOverflow::new(state.current_token.location));
                    None
                }
            }
            RawToken::FloatLiteral => {
                state.advance();

                if let Ok(value) = state
                    .resolve_current_token_str()
                    .replace('_', "")
                    .parse::<f64>()
                {
                    Some(Literal::Float {
                        value,
                        location: state.current_token.location,
                    })
                } else {
                    state
                        .diagnostics
                        .add_diagnostic(FloatOverflow::new(state.current_token.location));
                    None
                }
            }
            RawToken::StringLiteral => {
                state.advance();
                Some(Literal::String {
                    value: state.lexer.scanned_string(),
                    location: state.current_token.location,
                })
            }
            RawToken::CharLiteral => {
                state.advance();
                Some(Literal::Character {
                    value: state.lexer.scanned_char,
                    location: state.current_token.location,
                })
            }
            RawToken::TrueBoolLiteral => {
                state.advance();
                Some(Literal::Boolean {
                    value: true,
                    location: state.current_token.location,
                })
            }
            RawToken::FalseBoolLiteral => {
                state.advance();
                Some(Literal::Boolean {
                    value: false,
                    location: state.current_token.location,
                })
            }
            _ => {
                unreachable!()
            }
        }
    }
}
