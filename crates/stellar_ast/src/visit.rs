//! Provides a [`Visitor`] trait to traverse AST.
//!
//! In the [`Visitor`] trait, every method starts with the `visit_` and then
//! the AST node name.
//!
//! ```
//! use stellar_ast::{Expression, visit::Visitor};
//!
//! pub struct AllExpressionsPrinter;
//!
//! impl Visitor for AllExpressionsPrinter {
//!     fn visit_expression(&mut self, expression: &Expression) {
//!         println!("expression found: {:?}", expression);
//!     }
//! }
//! ```

use stellar_filesystem::location::Location;

use crate::{
    BinaryOperator, Enum, Expression, Function, GenericParameter, IdentifierAST, ImportPath,
    Interface, LambdaFunctionParameter, Literal, MatchExpressionItem, Module, ModuleItem, Path,
    Pattern, PostfixOperator, PrefixOperator, Statement, Struct, StructField,
    StructFieldExpression, StructFieldPattern, TupleField, TupleLikeStruct, Type, TypeAlias,
    TypeConstructor, WherePredicate,
};

/// Allows to traverse AST.
///
/// See [module level docs](crate::visit) for more details.
#[allow(unused_variables)]
pub trait Visitor {
    /// Visits a module.
    fn visit_module(&mut self, module: &Module) {
        for item in &module.items {
            self.visit_module_item(item);
        }
    }

    /// Visits a module item.
    fn visit_module_item(&mut self, module_item: &ModuleItem) {
        match module_item {
            ModuleItem::Enum(enum_) => self.visit_enum(enum_),
            ModuleItem::Interface(interface) => self.visit_interface(interface),
            ModuleItem::Function(function) => self.visit_function(function),
            ModuleItem::Import { location, path } => self.visit_import(*location, path),
            ModuleItem::Struct(struct_) => self.visit_struct(struct_),
            ModuleItem::TupleLikeStruct(tl_struct) => self.visit_tuple_like_struct(tl_struct),
            ModuleItem::TypeAlias(alias) => self.visit_type_alias(alias),
        }
    }

    /// Visits an import.
    fn visit_import(&mut self, location: Location, path: &ImportPath) {
        self.visit_import_path(path);
    }

    /// Visits an import path.
    fn visit_import_path(&mut self, path: &ImportPath) {}

    /// Visits an enum module item.
    fn visit_enum(&mut self, enum_: &Enum) {
        self.visit_generic_parameters(&enum_.generic_parameters);
        self.visit_where_predicates(&enum_.where_predicates);
        self.visit_methods(&enum_.methods);
        self.visit_implements(enum_.implements.as_deref());
    }

    /// Visits an interface module item.
    fn visit_interface(&mut self, interface: &Interface) {
        self.visit_generic_parameters(&interface.generic_parameters);
        self.visit_where_predicates(&interface.where_predicates);
        self.visit_methods(&interface.methods);
        self.visit_inherits(interface.inherits.as_deref());
    }

    /// Visits a struct module item.
    fn visit_struct(&mut self, struct_: &Struct) {
        self.visit_generic_parameters(&struct_.generic_parameters);
        self.visit_where_predicates(&struct_.where_predicates);
        self.visit_struct_fields(&struct_.fields);
        self.visit_methods(&struct_.methods);
        self.visit_implements(struct_.implements.as_deref());
    }

    /// Visits a tuple-like struct module item.
    fn visit_tuple_like_struct(&mut self, tl_struct: &TupleLikeStruct) {
        self.visit_generic_parameters(&tl_struct.generic_parameters);
        self.visit_where_predicates(&tl_struct.where_predicates);
        self.visit_tuple_fields(&tl_struct.fields);
        self.visit_methods(&tl_struct.methods);
        self.visit_implements(tl_struct.implements.as_deref());
    }

    /// Visits a type alias module item.
    fn visit_type_alias(&mut self, alias: &TypeAlias) {
        self.visit_generic_parameters(&alias.generic_parameters);
        self.visit_type(&alias.value);
    }

    /// Visits tuple fields.
    fn visit_tuple_fields(&mut self, fields: &[TupleField]) {
        for field in fields {
            self.visit_tuple_field(field);
        }
    }

    /// Visits a tuple field.
    fn visit_tuple_field(&mut self, field: &TupleField) {
        self.visit_type(&field.ty);
    }

    /// Visits struct fields.
    fn visit_struct_fields(&mut self, fields: &[StructField]) {
        for field in fields {
            self.visit_struct_field(field);
        }
    }

    /// Visits a struct field.
    fn visit_struct_field(&mut self, field: &StructField) {
        self.visit_type(&field.ty);
    }

    /// Visits generic parameters.
    fn visit_generic_parameters(&mut self, generic_parameters: &[GenericParameter]) {
        for generic_parameter in generic_parameters {
            self.visit_generic_parameter(generic_parameter);
        }
    }

    /// Visits a generic parameter.
    fn visit_generic_parameter(&mut self, generic_parameter: &GenericParameter) {
        if let Some(default_value) = &generic_parameter.default_value {
            self.visit_type(default_value);
        }

        if let Some(bounds) = &generic_parameter.bounds {
            self.visit_bounds(bounds);
        }
    }

    /// Visits where predicates.
    fn visit_where_predicates(&mut self, predicates: &[WherePredicate]) {
        for predicate in predicates {
            self.visit_where_predicate(predicate);
        }
    }

    /// Visits a where predicate.
    fn visit_where_predicate(&mut self, predicate: &WherePredicate) {
        self.visit_type(&predicate.ty);
        self.visit_bounds(&predicate.bounds);
    }

    /// Visits a function.
    fn visit_function(&mut self, function: &Function) {
        if let Some(body) = &function.body {
            self.visit_statements_block(body);
        }
    }

    /// Visits a method.
    fn visit_method(&mut self, method: &Function) {
        self.visit_function(method);
    }

    /// Visits methods.
    fn visit_methods(&mut self, methods: &[Function]) {
        for method in methods {
            self.visit_method(method);
        }
    }

    /// Visits interfaces, that a particular type implements.
    fn visit_implements(&mut self, implements: Option<&[TypeConstructor]>) {
        if let Some(implements) = implements {
            for interface in implements {
                self.visit_type_constructor(interface);
            }
        }
    }

    /// Visits interfaces, that a particular interface inherits.
    fn visit_inherits(&mut self, inherits: Option<&[TypeConstructor]>) {
        if let Some(inherits) = inherits {
            for interface in inherits {
                self.visit_type_constructor(interface);
            }
        }
    }

    /// Visits a statements block.
    fn visit_statements_block(&mut self, statements: &[Statement]) {
        for statement in statements {
            self.visit_statement(statement);
        }
    }

    /// Visits a statement.
    fn visit_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Break { location } => self.visit_break_statement(*location),
            Statement::Continue { location } => self.visit_continue_statement(*location),
            Statement::Defer { call } => self.visit_defer_expression(call),
            Statement::Expression {
                expression,
                has_semicolon,
            } => self.visit_expression_statement(expression, *has_semicolon),
            Statement::Let { pattern, value, ty } => {
                self.visit_let_statement(pattern, value, ty.as_ref());
            }
            Statement::Return { expression } => self.visit_return_statement(expression),
        }
    }

    /// Visits a break statement.
    fn visit_break_statement(&mut self, location: Location) {}

    /// Visits a continue statement.
    fn visit_continue_statement(&mut self, location: Location) {}

    /// Visits a defer expression.
    fn visit_defer_expression(&mut self, call: &Expression) {}

    /// Visits an expression statement.
    fn visit_expression_statement(&mut self, expression: &Expression, has_semicolon: bool) {
        self.visit_expression(expression);
    }

    /// Visits a let statement.
    fn visit_let_statement(&mut self, pattern: &Pattern, value: &Expression, ty: Option<&Type>) {
        self.visit_pattern(pattern);
        self.visit_expression(value);

        if let Some(ty) = ty {
            self.visit_type(ty);
        }
    }

    /// Visits a return statement.
    fn visit_return_statement(&mut self, expression: &Expression) {
        self.visit_expression(expression);
    }

    /// Visits a pattern.
    fn visit_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Grouped { location, inner } => self.visit_grouped_pattern(*location, inner),
            Pattern::Identifier {
                location,
                identifier,
                pattern,
            } => self.visit_identifier_pattern(*location, *identifier, pattern),
            Pattern::List {
                location,
                inner_patterns,
            } => {
                self.visit_list_pattern(*location, inner_patterns);
            }
            Pattern::Literal(literal) => self.visit_literal_pattern(literal),
            Pattern::Or {
                location,
                left,
                right,
            } => self.visit_or_pattern(left, right),
            Pattern::Path { path } => self.visit_path_pattern(path),
            Pattern::Rest { location } => self.visit_rest_pattern(*location),
            Pattern::Struct {
                location,
                path,
                fields,
            } => {
                self.visit_struct_pattern(*location, path, fields);
            }
            Pattern::Tuple { location, elements } => {
                self.visit_tuple_pattern(*location, elements);
            }
            Pattern::TupleLike {
                location,
                path,
                inner_patterns,
            } => {
                self.visit_tuple_like_pattern(*location, path, inner_patterns);
            }
            Pattern::Wildcard { location } => self.visit_wildcard_pattern(*location),
        }
    }

    /// Visits a grouped pattern.
    fn visit_grouped_pattern(&mut self, location: Location, inner: &Pattern) {
        self.visit_pattern(inner);
    }

    /// Visits an identifier pattern.
    fn visit_identifier_pattern(
        &mut self,
        location: Location,
        identifier: IdentifierAST,
        pattern: &Option<Box<Pattern>>,
    ) {
        if let Some(pattern) = pattern {
            self.visit_pattern(pattern);
        }
    }

    /// Visits a list pattern.
    fn visit_list_pattern(&mut self, location: Location, inner_patterns: &[Pattern]) {
        for pattern in inner_patterns {
            self.visit_pattern(pattern);
        }
    }

    /// Visits a literal pattern.
    fn visit_literal_pattern(&mut self, literal: &Literal) {}

    /// Visits an or pattern.
    fn visit_or_pattern(&mut self, left: &Pattern, right: &Pattern) {
        self.visit_pattern(left);
        self.visit_pattern(right);
    }

    /// Visits a path pattern.
    fn visit_path_pattern(&mut self, path: &Path) {}

    /// Visits a rest pattern.
    fn visit_rest_pattern(&mut self, location: Location) {}

    /// Visits a struct pattern.
    fn visit_struct_pattern(
        &mut self,
        location: Location,
        path: &Path,
        field_patterns: &[StructFieldPattern],
    ) {
        for field_pattern in field_patterns {
            self.visit_struct_field_pattern(field_pattern);
        }
    }

    /// Visits a struct field pattern.
    fn visit_struct_field_pattern(&mut self, field: &StructFieldPattern) {}

    /// Visits a tuple pattern.
    fn visit_tuple_pattern(&mut self, location: Location, elements: &[Pattern]) {}

    /// Visits a tuple-like pattern.
    fn visit_tuple_like_pattern(
        &mut self,
        location: Location,
        path: &Path,
        inner_patterns: &[Pattern],
    ) {
        for pattern in inner_patterns {
            self.visit_pattern(pattern);
        }
    }

    /// Visits a wildcard pattern.
    fn visit_wildcard_pattern(&mut self, location: Location) {}

    /// Visits a type.
    fn visit_type(&mut self, ty: &Type) {
        match ty {
            Type::Constructor(constructor) => self.visit_type_constructor(constructor),
            Type::Function {
                location,
                parameter_types,
                return_type,
            } => self.visit_function_type(*location, parameter_types, return_type.as_deref()),
            Type::InterfaceObject { location, bounds } => {
                self.visit_interface_object_type(*location, bounds);
            }
            Type::Parenthesized { location, inner } => {
                self.visit_parenthesized_type(*location, inner);
            }
            Type::Tuple {
                location,
                element_types,
            } => {
                self.visit_tuple_type(*location, element_types);
            }
            Type::Underscore { location } => self.visit_underscore_type(*location),
        }
    }

    /// Visits arguments in a type constructor.
    fn visit_type_arguments(&mut self, arguments: &[Type]) {
        for argument in arguments {
            self.visit_type(argument);
        }
    }

    /// Visits a type constructor.
    fn visit_type_constructor(&mut self, constructor: &TypeConstructor) {
        self.visit_type_arguments(&constructor.arguments);
    }

    /// Visits a function type.
    fn visit_function_type(
        &mut self,
        location: Location,
        parameter_types: &[Type],
        return_type: Option<&Type>,
    ) {
        for parameter_type in parameter_types {
            self.visit_type(parameter_type);
        }

        if let Some(return_type) = return_type {
            self.visit_type(return_type);
        }
    }

    /// Visits an interface object type.
    fn visit_interface_object_type(&mut self, location: Location, bounds: &[TypeConstructor]) {
        self.visit_bounds(bounds);
    }

    /// Visits type bounds.
    fn visit_bounds(&mut self, bounds: &[TypeConstructor]) {
        for bound in bounds {
            self.visit_type_constructor(bound);
        }
    }

    /// Visits a parenthesized type.
    fn visit_parenthesized_type(&mut self, location: Location, inner: &Type) {
        self.visit_type(inner);
    }

    /// Visits a tuple type.
    fn visit_tuple_type(&mut self, location: Location, element_types: &[Type]) {
        for element_type in element_types {
            self.visit_type(element_type);
        }
    }

    /// Visit an underscore type.
    fn visit_underscore_type(&mut self, location: Location) {}

    /// Visits an expression.
    fn visit_expression(&mut self, expression: &Expression) {
        match expression {
            Expression::As {
                location,
                left,
                right,
            } => {
                self.visit_as_expression(*location, left, right);
            }
            Expression::Binary {
                location,
                left,
                operator,
                right,
            } => {
                self.visit_binary_expression(*location, left, *operator, right);
            }
            Expression::Call {
                location,
                callee,
                arguments,
            } => {
                self.visit_call_expression(*location, callee, arguments);
            }
            Expression::FieldAccess {
                location,
                left,
                right,
            } => {
                self.visit_field_access_expression(*location, left, *right);
            }
            Expression::Identifier(identifier) => self.visit_identifier_expression(*identifier),
            Expression::List { location, elements } => {
                self.visit_list_expression(*location, elements);
            }
            Expression::Literal(literal) => self.visit_literal_expression(literal),
            Expression::If {
                location,
                if_blocks,
                r#else,
            } => {
                self.visit_if_expression(*location, if_blocks, r#else.as_deref());
            }
            Expression::Lambda {
                location,
                parameters,
                return_type,
                value,
            } => {
                self.visit_lambda_expression(*location, parameters, return_type.as_ref(), value);
            }
            Expression::Loop {
                location,
                statements_block,
            } => {
                self.visit_loop_expression(*location, statements_block);
            }
            Expression::Match {
                location,
                expression,
                block,
            } => {
                self.visit_match_expression(*location, expression, block);
            }
            Expression::Parenthesized { location, inner } => {
                self.visit_parenthesized_expression(*location, inner);
            }
            Expression::Postfix {
                location,
                inner,
                operator,
            } => {
                self.visit_postfix_expression(*location, inner, *operator);
            }
            Expression::Prefix {
                location,
                inner,
                operator,
            } => {
                self.visit_prefix_expression(*location, inner, *operator);
            }
            Expression::StatementsBlock { location, block } => {
                self.visit_statements_block_expression(*location, block);
            }
            Expression::Struct {
                location,
                left,
                fields,
            } => {
                self.visit_struct_expression(*location, left, fields);
            }
            Expression::Tuple { location, elements } => {
                self.visit_tuple_expression(*location, elements);
            }
            Expression::While {
                location,
                condition,
                statements_block,
            } => {
                self.visit_while_expression(*location, condition, statements_block);
            }
            Expression::TypeArguments {
                location,
                left,
                arguments,
            } => {
                self.visit_type_arguments_expression(*location, left, arguments);
            }
            Expression::Underscore { location } => self.visit_underscore_expression(*location),
        }
    }

    /// Visits an as expression.
    fn visit_as_expression(&mut self, location: Location, left: &Expression, right: &Type) {
        self.visit_expression(left);
        self.visit_type(right);
    }

    /// Visits a binary expression.
    fn visit_binary_expression(
        &mut self,
        location: Location,
        left: &Expression,
        operator: BinaryOperator,
        right: &Expression,
    ) {
        self.visit_expression(left);
        self.visit_expression(right);
    }

    /// Visits a call expression.
    fn visit_call_expression(
        &mut self,
        location: Location,
        callee: &Expression,
        arguments: &[Expression],
    ) {
        self.visit_expression(callee);

        for argument in arguments {
            self.visit_expression(argument);
        }
    }

    /// Visits a field access expression.
    fn visit_field_access_expression(
        &mut self,
        location: Location,
        left: &Expression,
        right: IdentifierAST,
    ) {
        self.visit_expression(left);
    }

    /// Visits an identifier expression.
    fn visit_identifier_expression(&mut self, identifier: IdentifierAST) {}

    /// Visits a list expression.
    fn visit_list_expression(&mut self, location: Location, elements: &[Expression]) {
        for element in elements {
            self.visit_expression(element);
        }
    }

    /// Visits a literal expression.
    fn visit_literal_expression(&mut self, literal: &Literal) {}

    /// Visits an if expression.
    fn visit_if_expression(
        &mut self,
        location: Location,
        if_blocks: &[(Expression, Vec<Statement>)],
        r#else: Option<&[Statement]>,
    ) {
        for (condition, block) in if_blocks {
            self.visit_expression(condition);
            self.visit_statements_block(block);
        }

        if let Some(r#else) = r#else {
            self.visit_statements_block(r#else);
        }
    }

    /// Visits a lambda expression.
    fn visit_lambda_expression(
        &mut self,
        location: Location,
        parameters: &[LambdaFunctionParameter],
        return_type: Option<&Type>,
        value: &Expression,
    ) {
        for parameter in parameters {
            self.visit_lambda_function_parameter(parameter);
        }

        if let Some(return_type) = return_type {
            self.visit_type(return_type);
        }

        self.visit_expression(value);
    }

    /// Visits an underscore expression.
    fn visit_underscore_expression(&mut self, location: Location) {}

    /// Visits a lambda function parameter.
    fn visit_lambda_function_parameter(&mut self, parameter: &LambdaFunctionParameter) {
        if let Some(ty) = &parameter.ty {
            self.visit_type(ty);
        }
    }

    /// Visits a loop expression.
    fn visit_loop_expression(&mut self, location: Location, statements_block: &[Statement]) {
        self.visit_statements_block(statements_block);
    }

    /// Visits a match expression.
    fn visit_match_expression(
        &mut self,
        location: Location,
        expression: &Expression,
        block: &[MatchExpressionItem],
    ) {
        self.visit_expression(expression);

        for item in block {
            self.visit_match_expression_item(item);
        }
    }

    /// Visits a match expression item.
    fn visit_match_expression_item(&mut self, item: &MatchExpressionItem) {
        self.visit_pattern(&item.left);
        self.visit_expression(&item.right);
    }

    /// Visits a parenthesized expression.
    fn visit_parenthesized_expression(&mut self, location: Location, inner: &Expression) {
        self.visit_expression(inner);
    }

    /// Visits a postfix expression.
    fn visit_postfix_expression(
        &mut self,
        location: Location,
        inner: &Expression,
        operator: PostfixOperator,
    ) {
        self.visit_expression(inner);
    }

    /// Visits a prefix expression.
    fn visit_prefix_expression(
        &mut self,
        location: Location,
        inner: &Expression,
        operator: PrefixOperator,
    ) {
        self.visit_expression(inner);
    }

    /// Visits a statements block expression.
    fn visit_statements_block_expression(&mut self, location: Location, block: &[Statement]) {
        self.visit_statements_block(block);
    }

    /// Visits a struct expression.
    fn visit_struct_expression(
        &mut self,
        location: Location,
        left: &Expression,
        fields: &[StructFieldExpression],
    ) {
        self.visit_expression(left);
        self.visit_struct_field_expressions(fields);
    }

    /// Visits struct field expressions.
    fn visit_struct_field_expressions(&mut self, fields: &[StructFieldExpression]) {
        for field in fields {
            self.visit_struct_field_expression(field);
        }
    }

    /// Visits a struct field expression.
    fn visit_struct_field_expression(&mut self, field: &StructFieldExpression) {
        if let Some(value) = &field.value {
            self.visit_expression(value);
        }
    }

    /// Visits a tuple expression.
    fn visit_tuple_expression(&mut self, location: Location, elements: &[Expression]) {
        for element in elements {
            self.visit_expression(element);
        }
    }

    /// Visits a while expression.
    fn visit_while_expression(
        &mut self,
        location: Location,
        condition: &Expression,
        statements_block: &[Statement],
    ) {
        self.visit_expression(condition);

        self.visit_statements_block(statements_block);
    }

    /// Visits type arguments expression.
    fn visit_type_arguments_expression(
        &mut self,
        location: Location,
        left: &Expression,
        arguments: &[Type],
    ) {
        self.visit_expression(left);
        self.visit_type_arguments(arguments);
    }
}
