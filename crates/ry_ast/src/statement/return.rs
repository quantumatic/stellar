use crate::expression::Expression;

use super::Statement;

#[derive(Debug, PartialEq)]
pub struct ReturnStatement {
    pub return_value: Expression,
}

impl From<ReturnStatement> for Statement {
    fn from(r#return: ReturnStatement) -> Self {
        Self::Return(r#return)
    }
}
