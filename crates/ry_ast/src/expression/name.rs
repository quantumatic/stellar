use string_interner::DefaultSymbol;

use super::RawExpression;

#[derive(Debug, PartialEq)]
pub struct IdentifierExpression {
    name: DefaultSymbol,
}

impl IdentifierExpression {
    #[inline]
    pub const fn new(name: DefaultSymbol) -> Self {
        Self { name }
    }

    #[inline]
    pub const fn name(&self) -> DefaultSymbol {
        self.name
    }
}

impl From<IdentifierExpression> for RawExpression {
    fn from(identifier: IdentifierExpression) -> Self {
        Self::Identifier(identifier)
    }
}
