use crate::expression::Expression;

use super::Statement;

#[derive(Debug, PartialEq)]
pub struct ReturnStatement {
    return_value: Expression,
}

impl ReturnStatement {
    #[inline]
    pub const fn new(return_value: Expression) -> Self {
        Self { return_value }
    }

    #[inline]
    pub const fn return_value(&self) -> &Expression {
        &self.return_value
    }
}

impl From<ReturnStatement> for Statement {
    fn from(r#return: ReturnStatement) -> Self {
        Self::Return(r#return)
    }
}
