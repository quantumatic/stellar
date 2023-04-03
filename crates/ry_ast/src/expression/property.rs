use crate::name::Name;

use super::{Expression, RawExpression};

#[derive(Debug, PartialEq)]
pub struct PropertyAccessExpression {
    pub left: Box<Expression>,
    pub right: Name,
}

impl PropertyAccessExpression {
    #[inline]
    pub fn new(left: Expression, right: Name) -> Self {
        Self {
            left: Box::new(left),
            right,
        }
    }
}

impl From<PropertyAccessExpression> for RawExpression {
    fn from(property_access: PropertyAccessExpression) -> Self {
        Self::Property(property_access)
    }
}
