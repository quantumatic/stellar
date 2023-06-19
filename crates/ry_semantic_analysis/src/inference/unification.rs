use ry_ast::{Type, TypeConstructor};
use std::{collections::HashMap, sync::Arc};

struct UnificationHandler {
    substitution: HashMap<usize, Arc<Type>>,
    next_unification_variable_index: usize,
}

struct InstanceKey {
    trait_name: String,
    type_name: String,
}

struct InstanceValue {
    generics: Vec<String>,
    module_name: String,
    trait_name: String,
}

impl UnificationHandler {
    fn new() -> Self {
        Self {
            substitution: HashMap::new(),
            next_unification_variable_index: 2,
        }
    }

    fn get(&mut self, index: usize) -> Option<Arc<Type>> {
        self.substitution.get(&index).map(|t| {
            match t.as_ref() {
                Type::Variable(i) => {
                    let mut to = None;

                    if let Some(t) = self.substitution.get(i) {
                        to = Some(t.clone());
                    } else {
                    }

                    if let Some(t) = to {
                        self.substitution.insert(index, t);
                    }
                }
                _ => {}
            }

            t.clone()
        })
    }

    fn substitute(&mut self, ty: Arc<Type>) -> Arc<Type> {
        match ty.as_ref() {
            Type::Variable(i) => {
                if let Some(t) = self.get(*i) {
                    self.substitute(t)
                } else {
                    ty
                }
            }
            Type::Constructor(constructor) => Arc::new(Type::Constructor(TypeConstructor {
                path: constructor.path,
                generic_arguments: constructor
                    .generic_arguments
                    .iter()
                    .map(|argument| self.substitute(argument.clone()))
                    .collect::<Vec<_>>(),
            })),
            Type::Tuple { element_types } => Arc::new(Type::Tuple {
                element_types: element_types
                    .iter()
                    .map(|element| self.substitute(element.clone()))
                    .collect(),
            }),
            Type::Function {
                parameter_types,
                return_type,
            } => Arc::new(Type::Function {
                parameter_types: parameter_types
                    .iter()
                    .map(|parameter| self.substitute(parameter.clone()))
                    .collect(),
                return_type: self.substitute(return_type.clone()),
            }),
        }
    }

    fn occurs_in_type(&self, var: usize, ty: Arc<Type>) -> bool {
        match ty.as_ref() {
            Type::Variable(var2) => var == *var2,
            Type::Function {
                parameter_types,
                return_type,
            } => {
                parameter_types.iter().any(|p| self.occurs_in_type(var, *p))
                    || self.occurs_in_type(var, *return_type)
            }
            _ => false,
        }
    }
}
