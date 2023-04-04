use super::RawExpression;
use ry_interner::Symbol;

#[derive(Debug, PartialEq)]
pub struct IdentifierExpression {
    pub name: Symbol,
}

impl From<IdentifierExpression> for RawExpression {
    fn from(identifier: IdentifierExpression) -> Self {
        Self::Identifier(identifier)
    }
}
