//! Defines `Expression` AST Node, as defined by the [spec].
use super::Statement;
use crate::expression::Expression;
use serde::{Deserialize, Serialize};

/// The `Expression` AST Node.
///
/// See the [module level documentation][self].
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ExpressionStatement {
    pub has_semicolon: bool,
    pub expression: Expression,
}

impl From<ExpressionStatement> for Statement {
    fn from(expression: ExpressionStatement) -> Self {
        Self::Expression(expression)
    }
}
