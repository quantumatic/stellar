use crate::expression::Expression;

use super::Statement;

#[derive(Debug, PartialEq)]
pub struct DeferStatement {
    value: Expression,
}

impl DeferStatement {
    #[inline]
    pub const fn value(&self) -> &Expression {
        &self.value
    }
}

impl From<DeferStatement> for Statement {
    fn from(defer: DeferStatement) -> Self {
        Self::Defer(defer)
    }
}
