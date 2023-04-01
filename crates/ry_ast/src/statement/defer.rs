use crate::expression::Expression;

use super::Statement;

#[derive(Debug, PartialEq)]
pub struct DeferStatement {
    call: Expression,
}

impl DeferStatement {
    #[inline]
    pub const fn new(call: Expression) -> Self {
        Self { call }
    }

    #[inline]
    pub const fn call(&self) -> &Expression {
        &self.call
    }
}

impl From<DeferStatement> for Statement {
    fn from(defer: DeferStatement) -> Self {
        Self::Defer(defer)
    }
}
