pub mod array;
pub mod negative_trait;
pub mod option;
pub mod primary;
pub mod reference;
pub mod where_clause;
pub mod generics;

use super::span::WithSpan;

use self::{
    array::ArrayType, negative_trait::NegativeTraitType, option::OptionType, primary::PrimaryType,
    reference::ReferenceType,
};

pub type Type = WithSpan<Box<RawType>>;

#[derive(Debug, PartialEq)]
pub enum RawType {
    Array(ArrayType),
    Reference(ReferenceType),
    Primary(PrimaryType),
    Option(OptionType),
    NegativeTrait(NegativeTraitType),
}
