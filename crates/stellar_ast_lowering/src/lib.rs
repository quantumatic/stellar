//! # AST Lowering
//!
//! AST Lowering is the process of converting AST into HIR.
//!
//! It:
//! * removes parenthesized expressions, types and grouped patterns.
//! * converts `loop {}` into `while true {}`.
//! * converts `interface A[T]: B[T] + C` into `interface A[T] where Self: B[T] + C`.
//! * converts `fun (A, B)` into `fun (A, B): ()` (adds unit type into function return types).
//!
//! See the [`stellar_hir`] crate for more details.

use diagnostics::{UnnecessaryGroupedPattern, UnnecessaryParenthesizedExpression};
use parking_lot::RwLock;
use stellar_ast::IdentifierAST;
use stellar_diagnostics::{BuildDiagnostic, Diagnostics};
use stellar_filesystem::location::{Location, DUMMY_LOCATION};
use stellar_interner::{builtin_identifiers::BIG_SELF, PathID};

mod diagnostics;

/// Provides [`LowerExt::lower()`] method.
pub trait LowerExt {
    /// Converts a given AST into HIR.
    #[must_use]
    fn lower(self, file_path_id: PathID, diagnostics: &RwLock<Diagnostics>) -> stellar_hir::Module;
}

impl LowerExt for stellar_ast::Module {
    #[inline(always)]
    fn lower(self, file_path_id: PathID, diagnostics: &RwLock<Diagnostics>) -> stellar_hir::Module {
        LoweringContext::new(file_path_id, diagnostics).lower(self)
    }
}

/// Represents a context for AST lowering.
///
/// See [crate-level docs](crate) for more details.
#[derive(Debug, Clone, Copy)]
struct LoweringContext<'d> {
    file_path_id: PathID,
    diagnostics: &'d RwLock<Diagnostics>,
}

impl<'d> LoweringContext<'d> {
    /// Creates a new lowering context.
    #[inline(always)]
    #[must_use]
    const fn new(file_path_id: PathID, diagnostics: &'d RwLock<Diagnostics>) -> Self {
        Self {
            file_path_id,
            diagnostics,
        }
    }

    #[inline(always)]
    fn add_diagnostic(&mut self, diagnostic: impl BuildDiagnostic) {
        self.diagnostics
            .write()
            .add_single_file_diagnostic(self.file_path_id, diagnostic);
    }

    /// Converts a given AST into HIR.
    #[must_use]
    pub fn lower(&mut self, ast: stellar_ast::Module) -> stellar_hir::Module {
        let mut lowered = stellar_hir::Module {
            items: vec![],
            docstring: ast.docstring,
        };

        for item in ast.items {
            lowered.items.push(self.lower_module_item(item));
        }

        lowered
    }

    /// Converts a given module item AST into HIR.
    pub fn lower_module_item(&mut self, ast: stellar_ast::ModuleItem) -> stellar_hir::ModuleItem {
        match ast {
            stellar_ast::ModuleItem::Enum(stellar_ast::Enum {
                visibility,
                name,
                generic_parameters,
                where_predicates,
                items,
                methods,
                implements,
                docstring,
            }) => stellar_hir::ModuleItem::Enum(stellar_hir::Enum {
                visibility,
                name,
                generic_parameters: self.lower_generic_parameters(generic_parameters),
                where_predicates: self.lower_where_predicates(where_predicates),
                items: items
                    .into_iter()
                    .map(|item| self.lower_enum_item(item))
                    .collect(),
                methods: methods
                    .into_iter()
                    .map(|method| self.lower_function(method))
                    .collect(),
                implements: implements.map(|implements| {
                    implements
                        .into_iter()
                        .map(|interface| self.lower_type_constructor(interface))
                        .collect()
                }),
                docstring,
            }),
            stellar_ast::ModuleItem::Struct(stellar_ast::Struct {
                visibility,
                name,
                generic_parameters,
                where_predicates,
                fields,
                methods,
                implements,
                docstring,
            }) => stellar_hir::ModuleItem::Struct(stellar_hir::Struct {
                visibility,
                name,
                generic_parameters: self.lower_generic_parameters(generic_parameters),
                where_predicates: self.lower_where_predicates(where_predicates),
                fields: fields
                    .into_iter()
                    .map(|field| self.lower_struct_field(field))
                    .collect(),
                methods: methods
                    .into_iter()
                    .map(|method| self.lower_function(method))
                    .collect(),
                implements: implements.map(|implements| {
                    implements
                        .into_iter()
                        .map(|interface| self.lower_type_constructor(interface))
                        .collect()
                }),
                docstring,
            }),
            stellar_ast::ModuleItem::Function(function) => {
                stellar_hir::ModuleItem::Function(self.lower_function(function))
            }
            stellar_ast::ModuleItem::Import { location, path } => {
                stellar_hir::ModuleItem::Import { location, path }
            }
            stellar_ast::ModuleItem::TypeAlias(alias) => {
                stellar_hir::ModuleItem::TypeAlias(self.lower_type_alias(alias))
            }
            stellar_ast::ModuleItem::TupleLikeStruct(stellar_ast::TupleLikeStruct {
                visibility,
                name,
                generic_parameters,
                where_predicates,
                fields,
                methods,
                implements,
                docstring,
            }) => stellar_hir::ModuleItem::TupleLikeStruct(stellar_hir::TupleLikeStruct {
                visibility,
                name,
                generic_parameters: self.lower_generic_parameters(generic_parameters),
                where_predicates: self.lower_where_predicates(where_predicates),
                fields: fields
                    .into_iter()
                    .map(|field| self.lower_tuple_field(field))
                    .collect(),
                methods: methods
                    .into_iter()
                    .map(|method| self.lower_function(method))
                    .collect(),
                implements: implements.map(|implements| {
                    implements
                        .into_iter()
                        .map(|interface| self.lower_type_constructor(interface))
                        .collect()
                }),
                docstring,
            }),
            stellar_ast::ModuleItem::Interface(stellar_ast::Interface {
                visibility,
                name,
                generic_parameters,
                where_predicates,
                methods,
                inherits,
                docstring,
            }) => stellar_hir::ModuleItem::Interface(stellar_hir::Interface {
                visibility,
                name,
                generic_parameters: self.lower_generic_parameters(generic_parameters),
                where_predicates: {
                    let mut where_predicates = self
                        .lower_where_predicates(where_predicates)
                        .into_iter()
                        .collect::<Vec<_>>();

                    if let Some(inherits) = inherits {
                        where_predicates.push(stellar_hir::WherePredicate {
                            ty: stellar_hir::Type::Constructor(stellar_hir::TypeConstructor {
                                location: name.location,
                                path: stellar_hir::Path {
                                    location: name.location,
                                    identifiers: vec![IdentifierAST {
                                        id: BIG_SELF,
                                        location: name.location,
                                    }],
                                },
                                arguments: vec![],
                            }),
                            bounds: inherits
                                .into_iter()
                                .map(|bound| self.lower_type_constructor(bound))
                                .collect(),
                        });
                    }

                    where_predicates
                },
                methods: methods
                    .into_iter()
                    .map(|method| self.lower_function(method))
                    .collect(),
                docstring,
            }),
        }
    }

    fn lower_function(&mut self, ast: stellar_ast::Function) -> stellar_hir::Function {
        stellar_hir::Function {
            signature: self.lower_function_signature(ast.signature),
            body: ast.body.map(|block| self.lower_statements_block(block)),
        }
    }

    fn lower_enum_item(&mut self, ast: stellar_ast::EnumItem) -> stellar_hir::EnumItem {
        match ast {
            stellar_ast::EnumItem::Just { name, docstring } => {
                stellar_hir::EnumItem::Just { name, docstring }
            }
            stellar_ast::EnumItem::Struct {
                name,
                fields,
                docstring,
            } => stellar_hir::EnumItem::Struct {
                name,
                fields: fields
                    .into_iter()
                    .map(|field| self.lower_struct_field(field))
                    .collect(),
                docstring,
            },
            stellar_ast::EnumItem::TupleLike {
                name,
                fields,
                docstring,
            } => stellar_hir::EnumItem::TupleLike {
                name,
                fields: fields
                    .into_iter()
                    .map(|field| self.lower_tuple_field(field))
                    .collect(),
                docstring,
            },
        }
    }
    fn lower_statements_block(
        &mut self,
        ast: Vec<stellar_ast::Statement>,
    ) -> Vec<stellar_hir::Statement> {
        ast.into_iter()
            .map(|statement| self.lower_statement(statement))
            .collect()
    }

    fn lower_statement(&mut self, ast: stellar_ast::Statement) -> stellar_hir::Statement {
        match ast {
            stellar_ast::Statement::Break { location } => {
                stellar_hir::Statement::Break { location }
            }
            stellar_ast::Statement::Continue { location } => {
                stellar_hir::Statement::Continue { location }
            }
            stellar_ast::Statement::Defer { call } => {
                let call = self.lower_expression(call);

                stellar_hir::Statement::Defer { call }
            }
            stellar_ast::Statement::Return { expression } => stellar_hir::Statement::Return {
                expression: self.lower_expression(expression),
            },
            stellar_ast::Statement::Let { pattern, value, ty } => stellar_hir::Statement::Let {
                pattern: self.lower_pattern(pattern),
                value: self.lower_expression(value),
                ty: ty.map(|ty| self.lower_type(ty)),
            },
            stellar_ast::Statement::Expression {
                expression,
                has_semicolon,
            } => stellar_hir::Statement::Expression {
                expression: self.lower_expression(expression),
                has_semicolon,
            },
        }
    }

    fn lower_pattern(&mut self, ast: stellar_ast::Pattern) -> stellar_hir::Pattern {
        match ast {
            stellar_ast::Pattern::Grouped { location, inner } => {
                self.add_diagnostic(UnnecessaryGroupedPattern::new(location));

                self.lower_pattern(*inner)
            }
            stellar_ast::Pattern::Wildcard { location } => {
                stellar_hir::Pattern::Wildcard { location }
            }
            stellar_ast::Pattern::Identifier {
                location,
                identifier,
                pattern,
            } => stellar_hir::Pattern::Identifier {
                location,
                identifier,
                pattern: pattern.map(|pattern| Box::new(self.lower_pattern(*pattern))),
            },
            stellar_ast::Pattern::List {
                location,
                inner_patterns,
            } => stellar_hir::Pattern::List {
                location,
                inner_patterns: inner_patterns
                    .into_iter()
                    .map(|pattern| self.lower_pattern(pattern))
                    .collect(),
            },
            stellar_ast::Pattern::Literal(literal) => stellar_hir::Pattern::Literal(literal),
            stellar_ast::Pattern::Or {
                location,
                left,
                right,
            } => stellar_hir::Pattern::Or {
                location,
                left: Box::new(self.lower_pattern(*left)),
                right: Box::new(self.lower_pattern(*right)),
            },
            stellar_ast::Pattern::Path { path } => stellar_hir::Pattern::Path { path },
            stellar_ast::Pattern::Rest { location } => stellar_hir::Pattern::Rest { location },
            stellar_ast::Pattern::Struct {
                location,
                path,
                fields,
            } => stellar_hir::Pattern::Struct {
                location,
                path,
                fields: fields
                    .into_iter()
                    .map(|field| self.lower_struct_field_pattern(field))
                    .collect(),
            },
            stellar_ast::Pattern::Tuple { location, elements } => stellar_hir::Pattern::Tuple {
                location,
                elements: elements
                    .into_iter()
                    .map(|element| self.lower_pattern(element))
                    .collect(),
            },
            stellar_ast::Pattern::TupleLike {
                location,
                path,
                inner_patterns,
            } => stellar_hir::Pattern::TupleLike {
                location,
                path,
                inner_patterns: inner_patterns
                    .into_iter()
                    .map(|pattern| self.lower_pattern(pattern))
                    .collect(),
            },
        }
    }

    fn lower_struct_field_pattern(
        &mut self,
        ast: stellar_ast::StructFieldPattern,
    ) -> stellar_hir::StructFieldPattern {
        match ast {
            stellar_ast::StructFieldPattern::NotRest {
                location,
                field_name,
                value_pattern,
            } => stellar_hir::StructFieldPattern::NotRest {
                location,
                field_name,
                value_pattern: value_pattern.map(|pattern| self.lower_pattern(pattern)),
            },
            stellar_ast::StructFieldPattern::Rest { location } => {
                stellar_hir::StructFieldPattern::Rest { location }
            }
        }
    }

    fn lower_expression(&mut self, ast: stellar_ast::Expression) -> stellar_hir::Expression {
        match ast {
            stellar_ast::Expression::Literal(literal) => stellar_hir::Expression::Literal(literal),
            stellar_ast::Expression::Identifier(identifier) => {
                stellar_hir::Expression::Identifier(identifier)
            }
            stellar_ast::Expression::Underscore { location } => {
                stellar_hir::Expression::Underscore { location }
            }
            stellar_ast::Expression::Loop {
                location,
                statements_block,
            } => stellar_hir::Expression::While {
                location,
                condition: Box::new(stellar_hir::Expression::Literal(
                    stellar_ast::Literal::Boolean {
                        value: true,
                        location,
                    },
                )),
                statements_block: self.lower_statements_block(statements_block),
            },
            stellar_ast::Expression::Tuple { location, elements } => {
                stellar_hir::Expression::Tuple {
                    location,
                    elements: elements
                        .into_iter()
                        .map(|element| self.lower_expression(element))
                        .collect(),
                }
            }
            stellar_ast::Expression::Lambda {
                location,
                parameters,
                return_type,
                value,
            } => stellar_hir::Expression::Lambda {
                location,
                parameters: parameters
                    .into_iter()
                    .map(|parameter| self.lower_lambda_function_parameter(parameter))
                    .collect(),
                return_type: return_type.map(|ty| self.lower_type(ty)),
                value: Box::new(self.lower_expression(*value)),
            },
            stellar_ast::Expression::Match {
                location,
                expression,
                block,
            } => {
                if let stellar_ast::Expression::Parenthesized { location, .. } = *expression {
                    self.add_diagnostic(UnnecessaryParenthesizedExpression::new(location));
                }

                stellar_hir::Expression::Match {
                    location,
                    expression: Box::new(self.lower_expression(*expression)),
                    block: block
                        .into_iter()
                        .map(|item| self.lower_match_expression_item(item))
                        .collect(),
                }
            }
            stellar_ast::Expression::Struct {
                location,
                left,
                fields,
            } => stellar_hir::Expression::Struct {
                location,
                left: Box::new(self.lower_expression(*left)),
                fields: fields
                    .into_iter()
                    .map(|field| self.lower_struct_field_expression(field))
                    .collect(),
            },
            stellar_ast::Expression::While {
                location,
                condition,
                statements_block: body,
            } => {
                if let stellar_ast::Expression::Parenthesized { location, .. } = *condition {
                    self.add_diagnostic(UnnecessaryParenthesizedExpression::new(location));
                }

                stellar_hir::Expression::While {
                    location,
                    condition: Box::new(self.lower_expression(*condition)),
                    statements_block: self.lower_statements_block(body),
                }
            }
            stellar_ast::Expression::Prefix {
                location,
                inner,
                operator,
            } => stellar_hir::Expression::Prefix {
                location,
                inner: Box::new(self.lower_expression(*inner)),
                operator,
            },
            stellar_ast::Expression::Postfix {
                location,
                inner,
                operator,
            } => stellar_hir::Expression::Postfix {
                location,
                inner: Box::new(self.lower_expression(*inner)),
                operator,
            },
            stellar_ast::Expression::If {
                location,
                if_blocks,
                r#else,
            } => stellar_hir::Expression::If {
                location,
                if_blocks: self.lower_if_blocks(if_blocks),
                r#else: r#else.map(|else_block| self.lower_statements_block(else_block)),
            },
            stellar_ast::Expression::Parenthesized { inner, .. } => {
                if let stellar_ast::Expression::Parenthesized { location, .. } = *inner {
                    self.add_diagnostic(UnnecessaryParenthesizedExpression::new(location));
                }

                self.lower_expression(*inner)
            }
            stellar_ast::Expression::Binary {
                location,
                left,
                right,
                operator,
            } => stellar_hir::Expression::Binary {
                location,
                left: Box::new(self.lower_expression(*left)),
                right: Box::new(self.lower_expression(*right)),
                operator,
            },
            stellar_ast::Expression::Call {
                location,
                callee,
                arguments,
            } => stellar_hir::Expression::Call {
                location,
                callee: Box::new(self.lower_expression(*callee)),
                arguments: arguments
                    .into_iter()
                    .map(|argument| self.lower_expression(argument))
                    .collect(),
            },
            stellar_ast::Expression::As {
                location,
                left,
                right,
            } => stellar_hir::Expression::As {
                location,
                left: Box::new(self.lower_expression(*left)),
                right: self.lower_type(right),
            },
            stellar_ast::Expression::List { location, elements } => stellar_hir::Expression::List {
                location,
                elements: elements
                    .into_iter()
                    .map(|element| self.lower_expression(element))
                    .collect(),
            },
            stellar_ast::Expression::FieldAccess {
                location,
                left,
                right,
            } => stellar_hir::Expression::FieldAccess {
                location,
                left: Box::new(self.lower_expression(*left)),
                right,
            },
            stellar_ast::Expression::TypeArguments {
                location,
                left,
                arguments,
            } => stellar_hir::Expression::TypeArguments {
                location,
                left: Box::new(self.lower_expression(*left)),
                type_arguments: self.lower_type_arguments(arguments),
            },
            stellar_ast::Expression::StatementsBlock { location, block } => {
                stellar_hir::Expression::StatementsBlock {
                    location,
                    block: self.lower_statements_block(block),
                }
            }
        }
    }

    fn lower_match_expression_item(
        &mut self,
        ast: stellar_ast::MatchExpressionItem,
    ) -> stellar_hir::MatchExpressionItem {
        if let stellar_ast::Expression::Parenthesized { location, .. } = ast.right {
            self.add_diagnostic(UnnecessaryParenthesizedExpression::new(location));
        }

        stellar_hir::MatchExpressionItem {
            left: self.lower_pattern(ast.left),
            right: self.lower_expression(ast.right),
        }
    }

    fn lower_struct_field_expression(
        &mut self,
        ast: stellar_ast::StructFieldExpression,
    ) -> stellar_hir::StructExpressionItem {
        stellar_hir::StructExpressionItem {
            name: ast.name,
            value: ast.value.map(|value| self.lower_expression(value)),
        }
    }

    fn lower_lambda_function_parameter(
        &mut self,
        ast: stellar_ast::LambdaFunctionParameter,
    ) -> stellar_hir::LambdaFunctionParameter {
        stellar_hir::LambdaFunctionParameter {
            name: ast.name,
            ty: ast.ty.map(|ty| self.lower_type(ty)),
        }
    }

    fn lower_if_blocks(
        &mut self,
        if_blocks: Vec<(stellar_ast::Expression, Vec<stellar_ast::Statement>)>,
    ) -> Vec<(stellar_hir::Expression, Vec<stellar_hir::Statement>)> {
        if_blocks
            .into_iter()
            .map(|if_block| self.lower_if_block(if_block))
            .collect()
    }

    fn lower_if_block(
        &mut self,
        if_block: (stellar_ast::Expression, Vec<stellar_ast::Statement>),
    ) -> (stellar_hir::Expression, Vec<stellar_hir::Statement>) {
        if let stellar_ast::Expression::Parenthesized { location, .. } = if_block.0 {
            self.add_diagnostic(UnnecessaryParenthesizedExpression::new(location));
        }

        (
            self.lower_expression(if_block.0),
            self.lower_statements_block(if_block.1),
        )
    }

    fn lower_type_arguments(&mut self, ast: Vec<stellar_ast::Type>) -> Vec<stellar_hir::Type> {
        ast.into_iter()
            .map(|type_argument| self.lower_type(type_argument))
            .collect()
    }

    fn lower_function_signature(
        &mut self,
        ast: stellar_ast::FunctionSignature,
    ) -> stellar_hir::FunctionSignature {
        stellar_hir::FunctionSignature {
            visibility: ast.visibility,
            name: ast.name,
            generic_parameters: self.lower_generic_parameters(ast.generic_parameters),
            parameters: ast
                .parameters
                .into_iter()
                .map(|parameter| self.lower_function_parameter(parameter))
                .collect(),
            return_type: ast.return_type.map(|ty| self.lower_type(ty)),
            where_predicates: self.lower_where_predicates(ast.where_predicates),
            docstring: ast.docstring,
        }
    }

    fn lower_function_parameter(
        &mut self,
        ast: stellar_ast::FunctionParameter,
    ) -> stellar_hir::FunctionParameter {
        match ast {
            stellar_ast::FunctionParameter::NotSelfParameter(parameter) => {
                stellar_hir::FunctionParameter::NotSelfParameter(
                    self.lower_not_self_function_parameter(parameter),
                )
            }
            stellar_ast::FunctionParameter::SelfParameter(parameter) => {
                stellar_hir::FunctionParameter::SelfParameter(
                    self.lower_self_function_parameter(parameter),
                )
            }
        }
    }

    fn lower_not_self_function_parameter(
        &mut self,
        ast: stellar_ast::NotSelfFunctionParameter,
    ) -> stellar_hir::NotSelfFunctionParameter {
        stellar_hir::NotSelfFunctionParameter {
            pattern: self.lower_pattern(ast.pattern),
            ty: self.lower_type(ast.ty),
        }
    }

    fn lower_self_function_parameter(
        &mut self,
        ast: stellar_ast::SelfFunctionParameter,
    ) -> stellar_hir::SelfFunctionParameter {
        stellar_hir::SelfFunctionParameter {
            self_location: ast.self_location,
            ty: ast.ty.map(|ty| self.lower_type(ty)),
        }
    }

    fn lower_type_alias(&mut self, ast: stellar_ast::TypeAlias) -> stellar_hir::TypeAlias {
        stellar_hir::TypeAlias {
            visibility: ast.visibility,
            name: ast.name,
            generic_parameters: self.lower_generic_parameters(ast.generic_parameters),
            value: self.lower_type(ast.value),
            docstring: ast.docstring,
        }
    }

    fn lower_struct_field(&mut self, ast: stellar_ast::StructField) -> stellar_hir::StructField {
        stellar_hir::StructField {
            visibility: ast.visibility,
            name: ast.name,
            ty: self.lower_type(ast.ty),
            docstring: ast.docstring,
        }
    }

    fn lower_tuple_field(&mut self, ast: stellar_ast::TupleField) -> stellar_hir::TupleField {
        stellar_hir::TupleField {
            visibility: ast.visibility,
            ty: self.lower_type(ast.ty),
        }
    }

    fn lower_generic_parameters(
        &mut self,
        ast: Vec<stellar_ast::GenericParameter>,
    ) -> Vec<stellar_hir::GenericParameter> {
        ast.into_iter()
            .map(|parameter| self.lower_generic_parameter(parameter))
            .collect()
    }

    fn lower_generic_parameter(
        &mut self,
        ast: stellar_ast::GenericParameter,
    ) -> stellar_hir::GenericParameter {
        stellar_hir::GenericParameter {
            name: ast.name,
            bounds: ast.bounds.map(|bounds| {
                bounds
                    .into_iter()
                    .map(|interface| self.lower_type_constructor(interface))
                    .collect()
            }),
            default_value: ast.default_value.map(|ty| self.lower_type(ty)),
        }
    }

    fn lower_where_predicates(
        &mut self,
        ast: Vec<stellar_ast::WherePredicate>,
    ) -> Vec<stellar_hir::WherePredicate> {
        ast.into_iter()
            .map(|item| self.lower_where_predicate(item))
            .collect()
    }

    fn lower_where_predicate(
        &mut self,
        ast: stellar_ast::WherePredicate,
    ) -> stellar_hir::WherePredicate {
        stellar_hir::WherePredicate {
            ty: self.lower_type(ast.ty),
            bounds: ast
                .bounds
                .into_iter()
                .map(|bound| self.lower_type_constructor(bound))
                .collect(),
        }
    }

    fn lower_type_constructor(
        &mut self,
        ast: stellar_ast::TypeConstructor,
    ) -> stellar_hir::TypeConstructor {
        stellar_hir::TypeConstructor {
            arguments: ast
                .arguments
                .into_iter()
                .map(|ty| self.lower_type(ty))
                .collect(),
            location: ast.location,
            path: ast.path,
        }
    }

    fn lower_underscore_type(&mut self, location: Location) -> stellar_hir::Type {
        stellar_hir::Type::Underscore { location }
    }

    fn lower_type(&mut self, ast: stellar_ast::Type) -> stellar_hir::Type {
        match ast {
            stellar_ast::Type::Function {
                location,
                parameter_types,
                return_type,
            } => stellar_hir::Type::Function {
                location,
                parameter_types: parameter_types
                    .into_iter()
                    .map(|ty| self.lower_type(ty))
                    .collect(),
                return_type: Box::new(return_type.map(|ty| self.lower_type(*ty)).unwrap_or(
                    stellar_hir::Type::Tuple {
                        location: DUMMY_LOCATION,
                        element_types: vec![],
                    },
                )),
            },
            stellar_ast::Type::Constructor(constructor) => {
                stellar_hir::Type::Constructor(self.lower_type_constructor(constructor))
            }
            stellar_ast::Type::Parenthesized { inner, .. } => {
                if let stellar_ast::Type::Parenthesized { location, .. } = *inner {
                    self.add_diagnostic(UnnecessaryParenthesizedExpression::new(location));
                }

                self.lower_type(*inner)
            }
            stellar_ast::Type::Underscore { location } => self.lower_underscore_type(location),
            stellar_ast::Type::InterfaceObject { location, bounds } => {
                stellar_hir::Type::InterfaceObject {
                    location,
                    bounds: bounds
                        .into_iter()
                        .map(|interface| self.lower_type_constructor(interface))
                        .collect(),
                }
            }
            stellar_ast::Type::Tuple {
                location,
                element_types,
            } => stellar_hir::Type::Tuple {
                location,
                element_types: element_types
                    .into_iter()
                    .map(|ty| self.lower_type(ty))
                    .collect(),
            },
        }
    }
}
