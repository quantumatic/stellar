use crate::expression::Expression;

use super::Statement;

#[derive(Debug, PartialEq)]
pub struct ReturnStatement {
    value: Expression,
}

impl ReturnStatement {
    #[inline]
    pub const fn value(&self) -> &Expression {
        &self.value
    }
}

impl From<ReturnStatement> for Statement {
    fn from(r#return: ReturnStatement) -> Self {
        Self::Return(r#return)
    }
}
