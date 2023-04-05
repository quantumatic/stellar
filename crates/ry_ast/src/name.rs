use crate::{
    span::{At, Spanned},
    token::{RawToken, Token},
};
use ry_interner::Symbol;

pub type Name = Spanned<Symbol>;

impl From<Token> for Option<Name> {
    fn from(token: Token) -> Self {
        match token.unwrap() {
            RawToken::Identifier(name) => Some((*name).at(token.span())),
            _ => None,
        }
    }
}

pub type Path = Spanned<Vec<Symbol>>;
