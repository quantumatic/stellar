use super::{Expression, RawExpression};
use crate::statement::StatementsBlock;

#[derive(Debug, PartialEq)]
pub struct IfElseBlock {
    condition: Expression,
    body: StatementsBlock,
}

impl IfElseBlock {
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
    if_else: Vec<IfElseBlock>,
    r#else: Option<StatementsBlock>,
}

impl IfExpression {
    #[inline]
    pub const fn if_else(&self) -> &Vec<IfElseBlock> {
        &self.if_else
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
