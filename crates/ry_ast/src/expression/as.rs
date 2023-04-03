use super::{Expression, RawExpression};
use crate::r#type::Type;

#[derive(Debug, PartialEq)]
pub struct AsExpression {
    left: Box<Expression>,
    right: Type,
}

impl AsExpression {
    #[inline]
    pub fn new(left: Expression, right: Type) -> Self {
        Self {
            left: Box::new(left),
            right,
        }
    }
}

impl From<AsExpression> for RawExpression {
    fn from(r#as: AsExpression) -> Self {
        Self::As(r#as)
    }
}
