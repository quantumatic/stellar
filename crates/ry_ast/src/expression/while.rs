use super::{Expression, RawExpression};
use crate::statement::StatementsBlock;

#[derive(Debug, PartialEq)]
pub struct WhileExpression {
    pub condition: Box<Expression>,
    pub body: StatementsBlock,
}

impl From<WhileExpression> for RawExpression {
    fn from(r#while: WhileExpression) -> Self {
        Self::While(r#while)
    }
}
