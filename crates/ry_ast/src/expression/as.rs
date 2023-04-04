use super::{Expression, RawExpression};
use crate::r#type::Type;

#[derive(Debug, PartialEq)]
pub struct AsExpression {
    pub left: Box<Expression>,
    pub right: Type,
}

impl From<AsExpression> for RawExpression {
    fn from(r#as: AsExpression) -> Self {
        Self::As(r#as)
    }
}
