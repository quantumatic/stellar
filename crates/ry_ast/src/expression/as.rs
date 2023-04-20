use super::{Expression, RawExpression};
use crate::r#type::Type;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AsExpression {
    pub left: Box<Expression>,
    pub right: Type,
}

impl From<AsExpression> for RawExpression {
    fn from(r#as: AsExpression) -> Self {
        Self::As(r#as)
    }
}
