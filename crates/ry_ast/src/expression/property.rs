use super::{Expression, RawExpression};
use crate::name::Name;

#[derive(Debug, PartialEq)]
pub struct PropertyAccessExpression {
    pub left: Box<Expression>,
    pub property: Name,
}

impl From<PropertyAccessExpression> for RawExpression {
    fn from(property_access: PropertyAccessExpression) -> Self {
        Self::Property(property_access)
    }
}