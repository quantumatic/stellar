use ry_diagnostics::GlobalDiagnostics;
use ry_hir::ty::{self, Type, TypeVariableID};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct TypeVariableGenerator {
    state: TypeVariableID,
}

impl TypeVariableGenerator {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self { state: 0 }
    }

    #[inline]
    #[must_use]
    pub fn next(&mut self) -> TypeVariableID {
        self.state += 1;
        self.state
    }

    #[inline]
    #[must_use]
    pub fn next_type(&mut self) -> ty::Type {
        ty::Type::Variable(self.next())
    }

    #[inline]
    #[must_use]
    pub const fn current(&self) -> TypeVariableID {
        self.state
    }
}

pub struct LoweringContext<'diagnostics> {
    type_variable_generator: TypeVariableGenerator,
    diagnostics: &'diagnostics mut GlobalDiagnostics,
}

impl<'diagnostics> LoweringContext<'diagnostics> {
    #[inline]
    #[must_use]
    pub fn new(
        type_variable_generator: TypeVariableGenerator,
        diagnostics: &'diagnostics mut GlobalDiagnostics,
    ) -> Self {
        Self {
            type_variable_generator,
            diagnostics,
        }
    }

    #[must_use]
    pub fn lower(&mut self, ast: ry_ast::Module) -> ry_hir::Module {
        let mut lowered = ry_hir::Module {
            items: vec![],
            docstring: ast.docstring,
        };

        for item in ast.items {
            lowered.items.push(self.lower_module_item(item));
        }

        lowered
    }

    pub fn lower_module_item(&mut self, ast: ry_ast::ModuleItem) -> ry_hir::ModuleItem {
        match ast {
            ry_ast::ModuleItem::Enum {
                visibility,
                name,
                generic_parameters,
                where_predicates,
                items,
                docstring,
            } => ry_hir::ModuleItem::Enum {
                visibility,
                name,
                generic_parameters: self.lower_generic_parameters(generic_parameters),
                where_predicates: self.lower_where_predicates(where_predicates),
                items: items
                    .into_iter()
                    .map(|item| self.lower_enum_item(item))
                    .collect(),
                docstring,
            },
            ry_ast::ModuleItem::Struct {
                visibility,
                name,
                generic_parameters,
                where_predicates,
                fields,
                docstring,
            } => ry_hir::ModuleItem::Struct {
                visibility,
                name,
                generic_parameters: self.lower_generic_parameters(generic_parameters),
                where_predicates: self.lower_where_predicates(where_predicates),
                fields: fields
                    .into_iter()
                    .map(|field| self.lower_struct_field(field))
                    .collect(),
                docstring,
            },
            ry_ast::ModuleItem::Impl(implementation) => {
                ry_hir::ModuleItem::Impl(self.lower_implementation(implementation))
            }
            ry_ast::ModuleItem::Function(function) => {
                ry_hir::ModuleItem::Function(self.lower_function(function))
            }
            ry_ast::ModuleItem::Import { location, path } => {
                ry_hir::ModuleItem::Import { location, path }
            }
            ry_ast::ModuleItem::TypeAlias(alias) => {
                ry_hir::ModuleItem::TypeAlias(self.lower_type_alias(alias))
            }
            ry_ast::ModuleItem::TupleLikeStruct {
                visibility,
                name,
                generic_parameters,
                where_predicates,
                fields,
                docstring,
            } => ry_hir::ModuleItem::TupleLikeStruct {
                visibility,
                name,
                generic_parameters: self.lower_generic_parameters(generic_parameters),
                where_predicates: self.lower_where_predicates(where_predicates),
                fields: fields
                    .into_iter()
                    .map(|field| self.lower_tuple_field(field))
                    .collect(),
                docstring,
            },
            ry_ast::ModuleItem::Trait {
                visibility,
                name,
                generic_parameters,
                where_predicates,
                items,
                docstring,
            } => ry_hir::ModuleItem::Trait {
                visibility,
                name,
                generic_parameters: self.lower_generic_parameters(generic_parameters),
                where_predicates: self.lower_where_predicates(where_predicates),
                items: items
                    .into_iter()
                    .map(|item| self.lower_trait_item(item))
                    .collect(),
                docstring,
            },
        }
    }

    fn lower_function(&mut self, ast: ry_ast::Function) -> ry_hir::Function {
        ry_hir::Function {
            signature: self.lower_function_signature(ast.signature),
            body: todo!(),
        }
    }

    fn lower_function_signature(
        &mut self,
        ast: ry_ast::FunctionSignature,
    ) -> ry_hir::FunctionSignature {
        ry_hir::FunctionSignature {
            visibility: ast.visibility,
            name: ast.name,
            generic_parameters: self.lower_generic_parameters(ast.generic_parameters),
            parameters: ast
                .parameters
                .into_iter()
                .map(|parameter| self.lower_function_parameter(parameter))
                .collect(),
            return_type: if ast.return_type.is_none() {
                Type::Unit
            } else {
                self.type_variable_generator.next_type()
            },
            return_type_expression: ast.return_type.map(|ty| self.lower_type_expression(ty)),
            where_predicates: self.lower_where_predicates(ast.where_predicates),
            docstring: ast.docstring,
        }
    }

    fn lower_function_parameter(
        &mut self,
        ast: ry_ast::FunctionParameter,
    ) -> ry_hir::FunctionParameter {
        match ast {
            ry_ast::FunctionParameter::NotSelfParameter(parameter) => {
                ry_hir::FunctionParameter::NotSelfParameter(
                    self.lower_not_self_function_parameter(parameter),
                )
            }
            ry_ast::FunctionParameter::SelfParameter(parameter) => {
                ry_hir::FunctionParameter::SelfParameter(
                    self.lower_self_function_parameter(parameter),
                )
            }
        }
    }

    fn lower_not_self_function_parameter(
        &mut self,
        ast: ry_ast::NotSelfFunctionParameter,
    ) -> ry_hir::NotSelfFunctionParameter {
        ry_hir::NotSelfFunctionParameter {
            name: ast.name,
            ty: self.lower_function_parameter_type(ast.ty),
        }
    }

    fn lower_function_parameter_type(
        &mut self,
        ast: ry_ast::FunctionParameterType,
    ) -> ry_hir::FunctionParameterType {
        match ast {
            ry_ast::FunctionParameterType::Type(ty) => ry_hir::FunctionParameterType::Type {
                ty: self.type_variable_generator.next_type(),
                type_expression: self.lower_type_expression(ty),
            },
            ry_ast::FunctionParameterType::Impl(bounds) => {
                ry_hir::FunctionParameterType::Impl(bounds)
            }
        }
    }

    fn lower_self_function_parameter(
        &mut self,
        ast: ry_ast::SelfFunctionParameter,
    ) -> ry_hir::SelfFunctionParameter {
        ry_hir::SelfFunctionParameter {
            self_location: ast.self_location,
            type_expression: ast.ty.map(|ty| self.lower_type_expression(ty)),
            ty: self.type_variable_generator.next_type(),
        }
    }

    fn lower_implementation(&mut self, ast: ry_ast::Impl) -> ry_hir::Impl {
        ry_hir::Impl {
            location: ast.location,
            generic_parameters: self.lower_generic_parameters(ast.generic_parameters),
            ty: self.lower_type_expression(ast.ty),
            r#trait: None,
            where_predicates: self.lower_where_predicates(ast.where_predicates),
            items: ast
                .items
                .into_iter()
                .map(|item| self.lower_trait_item(item))
                .collect(),
            docstring: None,
        }
    }

    fn lower_trait_item(&mut self, ast: ry_ast::TraitItem) -> ry_hir::TraitItem {
        match ast {
            ry_ast::TraitItem::TypeAlias(alias) => {
                ry_hir::TraitItem::TypeAlias(self.lower_type_alias(alias))
            }
            ry_ast::TraitItem::AssociatedFunction(function) => {
                ry_hir::TraitItem::AssociatedFunction(self.lower_function(function))
            }
        }
    }

    fn lower_type_alias(&mut self, ast: ry_ast::TypeAlias) -> ry_hir::TypeAlias {
        ry_hir::TypeAlias {
            visibility: ast.visibility,
            name: ast.name,
            generic_parameters: self.lower_generic_parameters(ast.generic_parameters),
            bounds: ast.bounds,
            value_type_expression: ast.value.map(|ty| self.lower_type_expression(ty)),
            value: self.type_variable_generator.next_type(),
            docstring: ast.docstring,
        }
    }

    fn lower_enum_item(&mut self, ast: ry_ast::EnumItem) -> ry_hir::EnumItem {
        match ast {
            ry_ast::EnumItem::Just { name, docstring } => {
                ry_hir::EnumItem::Just { name, docstring }
            }
            ry_ast::EnumItem::Struct {
                name,
                fields,
                docstring,
            } => ry_hir::EnumItem::Struct {
                name,
                fields: fields
                    .into_iter()
                    .map(|field| self.lower_struct_field(field))
                    .collect(),
                docstring,
            },
            ry_ast::EnumItem::TupleLike {
                name,
                fields,
                docstring,
            } => ry_hir::EnumItem::TupleLike {
                name,
                fields: fields
                    .into_iter()
                    .map(|field| self.lower_tuple_field(field))
                    .collect(),
                docstring,
            },
        }
    }

    fn lower_struct_field(&mut self, ast: ry_ast::StructField) -> ry_hir::StructField {
        ry_hir::StructField {
            visibility: ast.visibility,
            name: ast.name,
            type_expression: self.lower_type_expression(ast.ty),
            ty: self.type_variable_generator.next_type(),
            docstring: ast.docstring,
        }
    }

    fn lower_tuple_field(&mut self, ast: ry_ast::TupleField) -> ry_hir::TupleField {
        ry_hir::TupleField {
            visibility: ast.visibility,
            type_expression: self.lower_type_expression(ast.ty),
            ty: self.type_variable_generator.next_type(),
        }
    }

    fn lower_generic_parameters(
        &mut self,
        ast: Option<Vec<ry_ast::GenericParameter>>,
    ) -> Vec<ry_hir::GenericParameter> {
        ast.unwrap_or_else(|| {
            // todo: emit some diagnostics here

            vec![]
        })
        .into_iter()
        .map(|parameter| self.lower_generic_parameter(parameter))
        .collect()
    }

    fn lower_generic_parameter(
        &mut self,
        ast: ry_ast::GenericParameter,
    ) -> ry_hir::GenericParameter {
        ry_hir::GenericParameter {
            name: ast.name,
            bounds: ast.bounds,
            default_value_type_expression: ast
                .default_value
                .map(|ty| self.lower_type_expression(ty)),
            ty: self.type_variable_generator.next_type(),
        }
    }

    fn lower_where_predicates(
        &mut self,
        ast: Option<Vec<ry_ast::WherePredicate>>,
    ) -> Vec<ry_hir::WherePredicate> {
        ast.unwrap_or_else(|| {
            // todo: emit some diagnostics here

            vec![]
        })
        .into_iter()
        .map(|item| self.lower_where_predicate(item))
        .collect()
    }

    fn lower_where_predicate(&mut self, ast: ry_ast::WherePredicate) -> ry_hir::WherePredicate {
        match ast {
            ry_ast::WherePredicate::Eq { left, right } => ry_hir::WherePredicate::Eq {
                left_type_expression: self.lower_type_expression(left),
                left_ty: self.type_variable_generator.next_type(),
                right_type_expression: self.lower_type_expression(right),
                right_ty: self.type_variable_generator.next_type(),
            },
            ry_ast::WherePredicate::Satisfies { ty, bounds } => ry_hir::WherePredicate::Satisfies {
                ty: self.type_variable_generator.next_type(),
                type_expression: self.lower_type_expression(ty),
                bounds,
            },
        }
    }

    fn lower_type_expression(&mut self, ast: ry_ast::Type) -> ry_hir::TypeExpression {
        match ast {
            ry_ast::Type::Function {
                location,
                parameter_types,
                return_type,
            } => ry_hir::TypeExpression::Function {
                location,
                parameter_types: parameter_types
                    .into_iter()
                    .map(|ty| self.lower_type_expression(ty))
                    .collect(),
                return_type: Box::new(self.lower_type_expression(*return_type)),
            },
            ry_ast::Type::Path(path) => ry_hir::TypeExpression::Path(path),
            ry_ast::Type::Parenthesized { inner, .. } => {
                // todo: emit some diagnostics here

                self.lower_type_expression(*inner)
            }
            ry_ast::Type::TraitObject { location, bounds } => {
                ry_hir::TypeExpression::TraitObject { location, bounds }
            }
            ry_ast::Type::Tuple {
                location,
                element_types,
            } => ry_hir::TypeExpression::Tuple {
                location,
                element_types: element_types
                    .into_iter()
                    .map(|ty| self.lower_type_expression(ty))
                    .collect(),
            },
            ry_ast::Type::WithQualifiedPath {
                location,
                left,
                right,
                segments,
            } => ry_hir::TypeExpression::WithQualifiedPath {
                location,
                left: Box::new(self.lower_type_expression(*left)),
                right,
                segments,
            },
        }
    }
}
