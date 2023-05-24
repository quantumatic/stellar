mod array;
mod generics;
mod primary;
mod where_clause;

use serde::{Deserialize, Serialize};

pub use self::{
    array::ArrayType,
    generics::{Generic, Generics, TypeAnnotations},
    primary::PrimaryType,
    where_clause::{WhereClause, WhereClauseUnit},
};
use super::span::Spanned;

pub type Type = Spanned<RawType>;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum RawType {
    Array(ArrayType),
    Primary(PrimaryType),
}
