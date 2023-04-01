use super::{Expression, RawExpression};
use crate::statement::StatementsBlock;

#[derive(Debug, PartialEq)]
pub struct IfBlock {
    condition: Expression,
    body: StatementsBlock,
}

impl IfBlock {
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

#[derive(Debug, PartialEq)]
pub struct IfExpression {
    if_blocks: Vec<IfBlock>,
    r#else: Option<StatementsBlock>,
}

impl IfExpression {
    #[inline]
    pub const fn new(if_blocks: Vec<IfBlock>, r#else: Option<StatementsBlock>) -> Self {
        Self { if_blocks, r#else }
    }

    #[inline]
    pub const fn if_blocks(&self) -> &Vec<IfBlock> {
        &self.if_blocks
    }

    #[inline]
    pub const fn r#else(&self) -> Option<&StatementsBlock> {
        self.r#else.as_ref()
    }
}

impl From<IfExpression> for RawExpression {
    fn from(r#if: IfExpression) -> Self {
        Self::If(r#if)
    }
}
