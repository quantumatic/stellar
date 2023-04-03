use super::{Expression, RawExpression};
use crate::statement::StatementsBlock;

#[derive(Debug, PartialEq)]
pub struct IfBlock {
    pub condition: Expression,
    pub body: StatementsBlock,
}

impl IfBlock {
    #[inline]
    pub const fn new(condition: Expression, body: StatementsBlock) -> Self {
        Self { condition, body }
    }
}

#[derive(Debug, PartialEq)]
pub struct IfExpression {
    pub if_blocks: Vec<IfBlock>,
    pub r#else: Option<StatementsBlock>,
}

impl IfExpression {
    #[inline]
    pub const fn new(if_blocks: Vec<IfBlock>, r#else: Option<StatementsBlock>) -> Self {
        Self { if_blocks, r#else }
    }
}

impl From<IfExpression> for RawExpression {
    fn from(r#if: IfExpression) -> Self {
        Self::If(r#if)
    }
}
