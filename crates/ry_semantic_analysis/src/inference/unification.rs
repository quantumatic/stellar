use ry_ast::{Type, TypeVariable};

use super::InferenceContext;

impl InferenceContext {
    fn occurs_in_type(var: &TypeVariable, ty: &Type) -> bool {
        match ty {
            Type::Variable(var2) => var == var2,
            Type::Function {
                parameter_types,
                return_type,
            } => {
                parameter_types.iter().any(|p| Self::occurs_in_type(var, p))
                    || Self::occurs_in_type(var, return_type)
            }
            _ => false,
        }
    }
}
