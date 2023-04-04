use super::{Expression, RawExpression};
use crate::r#type::TypeAnnotations;

#[derive(Debug, PartialEq)]
pub struct TypeAnnotationsExpression {
    pub left: Box<Expression>,
    pub type_annotations: TypeAnnotations,
}

impl From<TypeAnnotationsExpression> for RawExpression {
    fn from(type_annotations: TypeAnnotationsExpression) -> Self {
        Self::TypeAnnotations(type_annotations)
    }
}
