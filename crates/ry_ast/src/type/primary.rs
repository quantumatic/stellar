use string_interner::DefaultSymbol;

use crate::span::WithSpan;

use super::{RawType, Type};

#[derive(Debug, PartialEq)]
pub struct PrimaryType {
    name: WithSpan<Vec<DefaultSymbol>>,
    generics: Vec<Type>,
}

impl PrimaryType {
    #[inline]
    pub const fn name(&self) -> &WithSpan<Vec<DefaultSymbol>> {
        &self.name
    }

    #[inline]
    pub const fn generics(&self) -> &Vec<Type> {
        &self.generics
    }
}

impl From<PrimaryType> for RawType {
    fn from(primary: PrimaryType) -> Self {
        Self::Primary(primary)
    }
}
