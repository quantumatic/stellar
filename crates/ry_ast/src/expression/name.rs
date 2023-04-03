use super::RawExpression;
use ry_interner::Symbol;

#[derive(Debug, PartialEq)]
pub struct IdentifierExpression {
    pub name: Symbol,
}

impl IdentifierExpression {
    #[inline]
    pub const fn new(name: Symbol) -> Self {
        Self { name }
    }
}

impl From<IdentifierExpression> for RawExpression {
    fn from(identifier: IdentifierExpression) -> Self {
        Self::Identifier(identifier)
    }
}
