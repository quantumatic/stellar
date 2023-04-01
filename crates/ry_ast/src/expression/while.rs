use super::{Expression, RawExpression};
use crate::statement::StatementsBlock;

#[derive(Debug, PartialEq)]
pub struct WhileExpression {
    condition: Expression,
    body: StatementsBlock,
}

impl WhileExpression {
    #[inline]
    pub const fn new(condition: Expression, body: StatementsBlock) -> Self {
        Self { condition, body }
    }

    #[inline]
    pub const fn condition(&self) -> &Expression {
        &self.condition
    }

    #[inline]
    pub const fn body(&self) -> &StatementsBlock {
        &self.body
    }
}

impl From<WhileExpression> for RawExpression {
    fn from(r#while: WhileExpression) -> Self {
        Self::While(r#while)
    }
}
