//! Defines `Expression` AST Node, as defined by the [spec].
use super::Statement;
use crate::expression::Expression;

/// The `Expression` AST Node.
///
/// See the [module level documentation][self].
#[derive(Debug, PartialEq)]
pub struct ExpressionStatement {
    pub has_semicolon: bool,
    pub expression: Expression,
}

impl From<ExpressionStatement> for Statement {
    fn from(expression: ExpressionStatement) -> Self {
        Self::Expression(expression)
    }
}
