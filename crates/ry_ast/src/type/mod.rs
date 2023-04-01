pub mod array;
pub mod generics;
pub mod option;
pub mod primary;
pub mod reference;
pub mod where_clause;

use super::span::WithSpan;

use self::{array::ArrayType, option::OptionType, primary::PrimaryType, reference::ReferenceType};

pub type Type = WithSpan<Box<RawType>>;
pub type TypeAnnotations = Vec<Type>;

#[derive(Debug, PartialEq)]
pub enum RawType {
    Array(ArrayType),
    Reference(ReferenceType),
    Primary(PrimaryType),
    Option(OptionType),
}
