use string_interner::DefaultSymbol;

use crate::{
    span::{WithSpan, WithSpannable},
    token::{RawToken, Token},
};

pub type Name = WithSpan<DefaultSymbol>;

impl From<Token> for Option<Name> {
    fn from(token: Token) -> Self {
        match token.unwrap() {
            RawToken::Identifier(name) => Some((*name).with_span(token.span())),
            _ => None,
        }
    }
}

pub type Path = WithSpan<Vec<DefaultSymbol>>;
