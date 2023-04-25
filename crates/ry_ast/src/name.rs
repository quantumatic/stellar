use crate::span::Spanned;
use ry_interner::Symbol;

pub type Name = Spanned<Symbol>;

pub type Path = Spanned<Vec<Name>>;
