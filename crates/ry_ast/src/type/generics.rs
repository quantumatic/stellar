use string_interner::DefaultSymbol;

use crate::span::WithSpan;

use super::Type;

pub type Generics = Vec<Generic>;

#[derive(Debug, PartialEq)]
pub struct Generic {
    name: WithSpan<DefaultSymbol>,
    constraint: Option<Type>,
}

impl Generic {
    #[inline]
    pub const fn name(&self) -> &WithSpan<DefaultSymbol> {
        &self.name
    }

    #[inline]
    pub const fn constraint(&self) -> Option<&Type> {
        self.constraint.as_ref()
    }
}

pub type TypeAnnotations = Vec<Type>;
