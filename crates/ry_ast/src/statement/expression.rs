use crate::expression::Expression;

use super::Statement;

#[derive(Debug, PartialEq)]
pub struct ExpressionStatement {
    has_semicolon: bool,
    expression: Expression,
}

impl ExpressionStatement {
    #[inline]
    pub const fn has_semicolon(&self) -> bool {
        self.has_semicolon
    }

    #[inline]
    pub const fn expression(&self) -> &Expression {
        &self.expression
    }
}

impl From<ExpressionStatement> for Statement {
    fn from(expression: ExpressionStatement) -> Self {
        Self::Expression(expression)
    }
}
