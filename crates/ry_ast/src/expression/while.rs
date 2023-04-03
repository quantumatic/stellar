use super::{Expression, RawExpression};
use crate::statement::StatementsBlock;

#[derive(Debug, PartialEq)]
pub struct WhileExpression {
    pub condition: Box<Expression>,
    pub body: StatementsBlock,
}

impl WhileExpression {
    #[inline]
    pub fn new(condition: Expression, body: StatementsBlock) -> Self {
        Self {
            condition: Box::new(condition),
            body,
        }
    }
}

impl From<WhileExpression> for RawExpression {
    fn from(r#while: WhileExpression) -> Self {
        Self::While(r#while)
    }
}
