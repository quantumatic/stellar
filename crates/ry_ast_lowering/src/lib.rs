use diagnostics::{UnnecessaryParenthesesInPatternDiagnostic, UnnecessaryParenthesizedExpression};
use ry_diagnostics::{BuildDiagnostic, GlobalDiagnostics};
use ry_filesystem::path_interner::PathID;
use ry_hir::ty::{self, Type, TypeVariableID};

mod diagnostics;

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
    pub fn generate_type_variable(&mut self) -> TypeVariableID {
        self.state += 1;
        self.state
    }

    #[inline]
    #[must_use]
    pub fn generate_type(&mut self) -> ty::Type {
        ty::Type::Variable(self.generate_type_variable())
    }

    #[inline]
    #[must_use]
    pub const fn current(&self) -> TypeVariableID {
        self.state
    }
}

pub struct LoweringContext<'diagnostics> {
    file_path_id: PathID,
    type_variable_generator: TypeVariableGenerator,
    diagnostics: &'diagnostics mut GlobalDiagnostics,
}

impl<'diagnostics> LoweringContext<'diagnostics> {
    #[inline]
    #[must_use]
    pub fn new(
        path_id: PathID,
        type_variable_generator: TypeVariableGenerator,
        diagnostics: &'diagnostics mut GlobalDiagnostics,
    ) -> Self {
        Self {
            file_path_id: path_id,
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
            body: ast.body.map(|block| self.lower_statements_block(block)),
        }
    }

    fn lower_statements_block(&mut self, ast: ry_ast::StatementsBlock) -> ry_hir::StatementsBlock {
        ast.into_iter()
            .map(|statement| self.lower_statement(statement))
            .collect()
    }

    fn lower_statement(&mut self, ast: ry_ast::Statement) -> ry_hir::Statement {
        match ast {
            ry_ast::Statement::Break { location } => ry_hir::Statement::Break { location },
            ry_ast::Statement::Continue { location } => ry_hir::Statement::Continue { location },
            ry_ast::Statement::Defer { call } => {
                let call = self.lower_expression(call);

                // todo: emit diagnostics if call is not call expression

                ry_hir::Statement::Defer { call }
            }
            ry_ast::Statement::Return { expression } => ry_hir::Statement::Return {
                expression: self.lower_expression(expression),
            },
            ry_ast::Statement::Let { pattern, value, ty } => ry_hir::Statement::Let {
                pattern: self.lower_pattern(pattern),
                value: self.lower_expression(value),
                type_expression: ty.map(|ty| self.lower_type_expression(ty)),
                ty: self.type_variable_generator.generate_type(),
            },
            ry_ast::Statement::Expression {
                expression,
                has_semicolon,
            } => ry_hir::Statement::Expression {
                expression: self.lower_expression(expression),
                has_semicolon,
            },
        }
    }

    fn lower_pattern(&mut self, ast: ry_ast::Pattern) -> ry_hir::Pattern {
        match ast {
            ry_ast::Pattern::Grouped { location, inner } => {
                // todo: emit diagnostics

                self.diagnostics.add_file_diagnostic(
                    [self.file_path_id],
                    UnnecessaryParenthesesInPatternDiagnostic { location }.build(),
                );

                self.lower_pattern(*inner)
            }
            ry_ast::Pattern::Identifier {
                location,
                identifier,
                pattern,
            } => ry_hir::Pattern::Identifier {
                location,
                identifier,
                pattern: pattern.map(|pattern| Box::new(self.lower_pattern(*pattern))),
                ty: self.type_variable_generator.generate_type(),
            },
            ry_ast::Pattern::List {
                location,
                inner_patterns,
            } => ry_hir::Pattern::List {
                location,
                inner_patterns: inner_patterns
                    .into_iter()
                    .map(|pattern| self.lower_pattern(pattern))
                    .collect(),
                ty: self.type_variable_generator.generate_type(),
            },
            ry_ast::Pattern::Literal(literal) => {
                ry_hir::Pattern::Literal(self.lower_literal(literal))
            }
            ry_ast::Pattern::Or {
                location,
                left,
                right,
            } => ry_hir::Pattern::Or {
                location,
                left: Box::new(self.lower_pattern(*left)),
                right: Box::new(self.lower_pattern(*right)),
                ty: self.type_variable_generator.generate_type(),
            },
            ry_ast::Pattern::Path { path } => ry_hir::Pattern::Path {
                path,
                ty: self.type_variable_generator.generate_type(),
            },
            ry_ast::Pattern::Rest { location } => ry_hir::Pattern::Rest {
                location,
                ty: self.type_variable_generator.generate_type(),
            },
            ry_ast::Pattern::Struct {
                location,
                path,
                fields,
            } => ry_hir::Pattern::Struct {
                location,
                path,
                fields: fields
                    .into_iter()
                    .map(|field| self.lower_struct_field_pattern(field))
                    .collect(),
                ty: self.type_variable_generator.generate_type(),
            },
            ry_ast::Pattern::Tuple { location, elements } => ry_hir::Pattern::Tuple {
                location,
                elements: elements
                    .into_iter()
                    .map(|element| self.lower_pattern(element))
                    .collect(),
                ty: self.type_variable_generator.generate_type(),
            },
            ry_ast::Pattern::TupleLike {
                location,
                path,
                inner_patterns,
            } => ry_hir::Pattern::TupleLike {
                location,
                path,
                inner_patterns: inner_patterns
                    .into_iter()
                    .map(|pattern| self.lower_pattern(pattern))
                    .collect(),
                ty: self.type_variable_generator.generate_type(),
            },
        }
    }

    fn lower_struct_field_pattern(
        &mut self,
        ast: ry_ast::StructFieldPattern,
    ) -> ry_hir::StructFieldPattern {
        match ast {
            ry_ast::StructFieldPattern::NotRest {
                location,
                field_name,
                value_pattern,
            } => ry_hir::StructFieldPattern::NotRest {
                location,
                field_name,
                value_pattern: value_pattern.map(|pattern| self.lower_pattern(pattern)),
                ty: self.type_variable_generator.generate_type(),
            },
            ry_ast::StructFieldPattern::Rest { location } => ry_hir::StructFieldPattern::Rest {
                location,
                ty: self.type_variable_generator.generate_type(),
            },
        }
    }

    fn lower_expression(&mut self, ast: ry_ast::Expression) -> ry_hir::Expression {
        match ast {
            ry_ast::Expression::Literal(literal) => {
                ry_hir::Expression::Literal(self.lower_literal(literal))
            }
            ry_ast::Expression::Identifier(identifier) => {
                ry_hir::Expression::Identifier(identifier)
            }
            ry_ast::Expression::Tuple { location, elements } => ry_hir::Expression::Tuple {
                location,
                elements: elements
                    .into_iter()
                    .map(|element| self.lower_expression(element))
                    .collect(),
                ty: self.type_variable_generator.generate_type(),
            },
            ry_ast::Expression::Lambda {
                location,
                parameters,
                return_type,
                block,
            } => ry_hir::Expression::Lambda {
                location,
                parameters: parameters
                    .into_iter()
                    .map(|parameter| self.lower_lambda_function_parameter(parameter))
                    .collect(),
                return_type_expression: return_type.map(|ty| self.lower_type_expression(ty)),
                block: self.lower_statements_block(block),
                ty: self.type_variable_generator.generate_type(),
            },
            ry_ast::Expression::Match {
                location,
                expression,
                block,
            } => {
                if let ry_ast::Expression::Parenthesized { location, .. } = *expression {
                    self.diagnostics.add_file_diagnostic(
                        [self.file_path_id],
                        UnnecessaryParenthesizedExpression { location }.build(),
                    );
                }

                ry_hir::Expression::Match {
                    location,
                    expression: Box::new(self.lower_expression(*expression)),
                    block: block
                        .into_iter()
                        .map(|item| self.lower_match_expression_item(item))
                        .collect(),
                    ty: self.type_variable_generator.generate_type(),
                }
            }
            ry_ast::Expression::Struct {
                location,
                left,
                fields,
            } => ry_hir::Expression::Struct {
                location,
                left: Box::new(self.lower_expression(*left)),
                fields: fields
                    .into_iter()
                    .map(|field| self.lower_struct_expression_item(field))
                    .collect(),
                ty: self.type_variable_generator.generate_type(),
            },
            ry_ast::Expression::While {
                location,
                condition,
                body,
            } => {
                if let ry_ast::Expression::Parenthesized { location, .. } = *condition {
                    self.diagnostics.add_file_diagnostic(
                        [self.file_path_id],
                        UnnecessaryParenthesizedExpression { location }.build(),
                    );
                }

                ry_hir::Expression::While {
                    location,
                    condition: Box::new(self.lower_expression(*condition)),
                    body: self.lower_statements_block(body),
                    ty: self.type_variable_generator.generate_type(),
                }
            }
            ry_ast::Expression::Prefix {
                location,
                inner,
                operator,
            } => ry_hir::Expression::Prefix {
                location,
                inner: Box::new(self.lower_expression(*inner)),
                operator,
                ty: self.type_variable_generator.generate_type(),
            },
            ry_ast::Expression::Postfix {
                location,
                inner,
                operator,
            } => ry_hir::Expression::Postfix {
                location,
                inner: Box::new(self.lower_expression(*inner)),
                operator,
                ty: self.type_variable_generator.generate_type(),
            },
            ry_ast::Expression::If {
                location,
                if_blocks,
                r#else,
            } => ry_hir::Expression::If {
                location,
                if_blocks: self.lower_if_blocks(if_blocks),
                r#else: r#else.map(|else_block| self.lower_statements_block(else_block)),
                ty: self.type_variable_generator.generate_type(),
            },
            ry_ast::Expression::Parenthesized { inner, .. } => {
                if let ry_ast::Expression::Parenthesized { location, .. } = *inner {
                    self.diagnostics.add_file_diagnostic(
                        [self.file_path_id],
                        UnnecessaryParenthesizedExpression { location }.build(),
                    );
                }

                self.lower_expression(*inner)
            }
            ry_ast::Expression::Binary {
                location,
                left,
                right,
                operator,
            } => ry_hir::Expression::Binary {
                location,
                left: Box::new(self.lower_expression(*left)),
                right: Box::new(self.lower_expression(*right)),
                operator,
                ty: self.type_variable_generator.generate_type(),
            },
            ry_ast::Expression::Call {
                location,
                callee,
                arguments,
            } => ry_hir::Expression::Call {
                location,
                callee: Box::new(self.lower_expression(*callee)),
                arguments: arguments
                    .into_iter()
                    .map(|argument| self.lower_expression(argument))
                    .collect(),
                ty: self.type_variable_generator.generate_type(),
            },
            ry_ast::Expression::As {
                location,
                left,
                right,
            } => ry_hir::Expression::As {
                location,
                left: Box::new(self.lower_expression(*left)),
                right: self.lower_type_expression(right),
                ty: self.type_variable_generator.generate_type(),
            },
            ry_ast::Expression::List { location, elements } => ry_hir::Expression::List {
                location,
                elements: elements
                    .into_iter()
                    .map(|element| self.lower_expression(element))
                    .collect(),
                ty: self.type_variable_generator.generate_type(),
            },
            ry_ast::Expression::FieldAccess {
                location,
                left,
                right,
            } => ry_hir::Expression::FieldAccess {
                location,
                left: Box::new(self.lower_expression(*left)),
                right,
                ty: self.type_variable_generator.generate_type(),
            },
            ry_ast::Expression::GenericArguments {
                location,
                left,
                generic_arguments,
            } => ry_hir::Expression::GenericArguments {
                location,
                left: Box::new(self.lower_expression(*left)),
                generic_arguments: self.lower_generic_arguments(generic_arguments),
                ty: self.type_variable_generator.generate_type(),
            },
            ry_ast::Expression::StatementsBlock { location, block } => {
                ry_hir::Expression::StatementsBlock {
                    location,
                    block: self.lower_statements_block(block),
                    ty: self.type_variable_generator.generate_type(),
                }
            }
        }
    }

    fn lower_match_expression_item(
        &mut self,
        ast: ry_ast::MatchExpressionItem,
    ) -> ry_hir::MatchExpressionItem {
        if let ry_ast::Expression::Parenthesized { location, .. } = ast.right {
            self.diagnostics.add_file_diagnostic(
                [self.file_path_id],
                UnnecessaryParenthesizedExpression { location }.build(),
            );
        }

        ry_hir::MatchExpressionItem {
            left: self.lower_pattern(ast.left),
            right: self.lower_expression(ast.right),
        }
    }

    fn lower_struct_expression_item(
        &mut self,
        ast: ry_ast::StructExpressionItem,
    ) -> ry_hir::StructExpressionItem {
        ry_hir::StructExpressionItem {
            name: ast.name,
            value: ast.value.map(|value| self.lower_expression(value)),
            ty: self.type_variable_generator.generate_type(),
        }
    }

    fn lower_lambda_function_parameter(
        &mut self,
        ast: ry_ast::LambdaFunctionParameter,
    ) -> ry_hir::LambdaFunctionParameter {
        ry_hir::LambdaFunctionParameter {
            name: ast.name,
            type_expression: ast.ty.map(|ty| self.lower_type_expression(ty)),
            ty: self.type_variable_generator.generate_type(),
        }
    }

    fn lower_if_blocks(
        &mut self,
        if_blocks: Vec<(ry_ast::Expression, ry_ast::StatementsBlock)>,
    ) -> Vec<(ry_hir::Expression, ry_hir::StatementsBlock)> {
        if_blocks
            .into_iter()
            .map(|if_block| self.lower_if_block(if_block))
            .collect()
    }

    fn lower_if_block(
        &mut self,
        if_block: (ry_ast::Expression, ry_ast::StatementsBlock),
    ) -> (ry_hir::Expression, ry_hir::StatementsBlock) {
        if let ry_ast::Expression::Parenthesized { location, .. } = if_block.0 {
            self.diagnostics.add_file_diagnostic(
                [self.file_path_id],
                UnnecessaryParenthesizedExpression { location }.build(),
            );
        }

        (
            self.lower_expression(if_block.0),
            self.lower_statements_block(if_block.1),
        )
    }

    fn lower_literal(&mut self, ast: ry_ast::Literal) -> ry_hir::Literal {
        ry_hir::Literal {
            literal: ast,
            ty: self.type_variable_generator.generate_type(),
        }
    }

    fn lower_generic_arguments(
        &mut self,
        ast: Vec<ry_ast::GenericArgument>,
    ) -> Vec<ry_hir::GenericArgument> {
        ast.into_iter()
            .map(|generic_argument| self.lower_generic_argument(generic_argument))
            .collect()
    }

    fn lower_generic_argument(&mut self, ast: ry_ast::GenericArgument) -> ry_hir::GenericArgument {
        match ast {
            ry_ast::GenericArgument::Type(ty) => ry_hir::GenericArgument::Type {
                type_expression: self.lower_type_expression(ty),
                ty: self.type_variable_generator.generate_type(),
            },
            ry_ast::GenericArgument::AssociatedType { name, value } => {
                ry_hir::GenericArgument::AssociatedType {
                    name,
                    value_type_expression: self.lower_type_expression(value),
                    value: self.type_variable_generator.generate_type(),
                }
            }
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
                self.type_variable_generator.generate_type()
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
                ty: self.type_variable_generator.generate_type(),
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
            ty: self.type_variable_generator.generate_type(),
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
            value: self.type_variable_generator.generate_type(),
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
            ty: self.type_variable_generator.generate_type(),
            docstring: ast.docstring,
        }
    }

    fn lower_tuple_field(&mut self, ast: ry_ast::TupleField) -> ry_hir::TupleField {
        ry_hir::TupleField {
            visibility: ast.visibility,
            type_expression: self.lower_type_expression(ast.ty),
            ty: self.type_variable_generator.generate_type(),
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
            ty: self.type_variable_generator.generate_type(),
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
                left_ty: self.type_variable_generator.generate_type(),
                right_type_expression: self.lower_type_expression(right),
                right_ty: self.type_variable_generator.generate_type(),
            },
            ry_ast::WherePredicate::Satisfies { ty, bounds } => ry_hir::WherePredicate::Satisfies {
                ty: self.type_variable_generator.generate_type(),
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
                if let ry_ast::Type::Parenthesized { location, .. } = *inner {
                    self.diagnostics.add_file_diagnostic(
                        [self.file_path_id],
                        UnnecessaryParenthesizedExpression { location }.build(),
                    );
                }

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
