use super::RawExpression;
use ry_interner::Symbol;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IdentifierExpression {
    pub name: Symbol,
}

impl From<IdentifierExpression> for RawExpression {
    fn from(identifier: IdentifierExpression) -> Self {
        Self::Identifier(identifier)
    }
}
