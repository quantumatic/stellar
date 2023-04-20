mod array;
mod generics;
mod primary;
mod reference;
mod where_clause;

use serde::{Deserialize, Serialize};

pub use self::{
    array::ArrayType,
    generics::{Generic, Generics, TypeAnnotations},
    primary::PrimaryType,
    reference::ReferenceType,
    where_clause::{WhereClause, WhereClauseUnit},
};
use super::span::Spanned;

pub type Type = Spanned<RawType>;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum RawType {
    Array(ArrayType),
    Reference(ReferenceType),
    Primary(PrimaryType),
}
