use crate::{
    span::{Spanned, WithSpan},
    token::{RawToken, Token},
};
use ry_interner::Symbol;

pub type Name = Spanned<Symbol>;

impl From<Token> for Option<Name> {
    fn from(token: Token) -> Self {
        match token.unwrap() {
            RawToken::Identifier(name) => Some((*name).with_span(token.span())),
            _ => None,
        }
    }
}

pub type Path = Spanned<Vec<Symbol>>;
