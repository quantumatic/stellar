pub mod array;
pub mod generics;
pub mod option;
pub mod primary;
pub mod reference;
pub mod where_clause;

pub use self::{
    array::ArrayType,
    generics::{Generic, Generics},
    option::OptionType,
    primary::PrimaryType,
    reference::ReferenceType,
    where_clause::{WhereClause, WhereClauseUnit}
};
use super::span::Spanned;

pub type Type = Spanned<RawType>;
pub type TypeAnnotations = Vec<Type>;

#[derive(Debug, PartialEq)]
pub enum RawType {
    Array(ArrayType),
    Reference(ReferenceType),
    Primary(PrimaryType),
    Option(OptionType),
}
