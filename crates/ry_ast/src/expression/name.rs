use string_interner::DefaultSymbol;

use super::RawExpression;

#[derive(Debug, PartialEq)]
pub struct IdentifierExpression {
    pub name: DefaultSymbol,
}

impl IdentifierExpression {
    #[inline]
    pub const fn new(name: DefaultSymbol) -> Self {
        Self { name }
    }
}

impl From<IdentifierExpression> for RawExpression {
    fn from(identifier: IdentifierExpression) -> Self {
        Self::Identifier(identifier)
    }
}
