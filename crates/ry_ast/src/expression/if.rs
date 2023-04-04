use super::{Expression, RawExpression};
use crate::statement::StatementsBlock;

#[derive(Debug, PartialEq)]
pub struct IfBlock {
    pub condition: Expression,
    pub body: StatementsBlock,
}

#[derive(Debug, PartialEq)]
pub struct IfExpression {
    pub if_blocks: Vec<IfBlock>,
    pub r#else: Option<StatementsBlock>,
}

impl From<IfExpression> for RawExpression {
    fn from(r#if: IfExpression) -> Self {
        Self::If(r#if)
    }
}
