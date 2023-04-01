use super::{Expression, RawExpression};
use crate::r#type::Type;

#[derive(Debug, PartialEq)]
pub struct AsExpression {
    left: Expression,
    right: Type,
}

impl AsExpression {
    #[inline]
    pub const fn new(left: Expression, right: Type) -> Self {
        Self { left, right }
    }

    #[inline]
    pub const fn left(&self) -> &Expression {
        &self.left
    }

    #[inline]
    pub const fn right(&self) -> &Type {
        &self.right
    }
}

impl From<AsExpression> for RawExpression {
    fn from(r#as: AsExpression) -> Self {
        Self::As(r#as)
    }
}
