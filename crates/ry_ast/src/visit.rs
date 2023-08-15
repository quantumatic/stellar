use ry_filesystem::location::Location;

use crate::{
    BinaryOperator, EnumItem, Expression, Function, GenericParameter, IdentifierAST, ImportPath,
    LambdaFunctionParameter, Literal, MatchExpressionItem, Module, ModuleItem, Path, Pattern,
    PostfixOperator, PrefixOperator, Statement, StructField, StructFieldExpression,
    StructFieldPattern, TupleField, Type, TypeAlias, TypeConstructor, Visibility, WherePredicate,
};

#[allow(unused_variables, clippy::too_many_arguments)]
pub trait Visitor {
    fn visit_module(&mut self, module: &Module) {
        for item in &module.items {
            self.visit_module_item(item);
        }
    }

    fn visit_module_item(&mut self, module_item: &ModuleItem) {
        match module_item {
            ModuleItem::Enum {
                visibility,
                name,
                generic_parameters,
                where_predicates,
                items,
                methods,
                implements,
                docstring,
            } => self.visit_enum(
                *visibility,
                *name,
                generic_parameters,
                where_predicates,
                items,
                methods,
                implements.as_deref(),
                docstring.as_deref(),
            ),
            ModuleItem::Interface {
                visibility,
                name,
                generic_parameters,
                where_predicates,
                methods,
                inherits,
                docstring,
            } => self.visit_interface(
                *visibility,
                *name,
                generic_parameters,
                where_predicates,
                methods,
                inherits.as_deref(),
                docstring.as_deref(),
            ),
            ModuleItem::Function(function) => self.visit_function(function),
            ModuleItem::Import { location, path } => self.visit_import(*location, path),
            ModuleItem::Struct {
                visibility,
                name,
                generic_parameters,
                where_predicates,
                fields,
                methods,
                implements,
                docstring,
            } => self.visit_struct(
                *visibility,
                *name,
                generic_parameters,
                where_predicates,
                fields,
                methods,
                implements.as_deref(),
                docstring.as_deref(),
            ),
            ModuleItem::TupleLikeStruct {
                visibility,
                name,
                generic_parameters,
                where_predicates,
                fields,
                methods,
                implements,
                docstring,
            } => self.visit_tuple_like_struct(
                *visibility,
                *name,
                generic_parameters,
                where_predicates,
                fields,
                methods,
                implements.as_deref(),
                docstring.as_deref(),
            ),
            ModuleItem::TypeAlias(alias) => self.visit_type_alias(alias),
        }
    }

    fn visit_import(&mut self, location: Location, path: &ImportPath) {
        self.visit_import_path(path);
    }

    fn visit_import_path(&mut self, path: &ImportPath) {}

    fn visit_enum(
        &mut self,
        visibility: Visibility,
        name: IdentifierAST,
        generic_parameters: &[GenericParameter],
        where_predicates: &[WherePredicate],
        items: &[EnumItem],
        methods: &[Function],
        implements: Option<&[TypeConstructor]>,
        docstring: Option<&str>,
    ) {
        self.visit_generic_parameters(generic_parameters);
        self.visit_where_predicates(where_predicates);
        self.visit_methods(methods);
        self.visit_implements(implements);
    }

    fn visit_interface(
        &mut self,
        visibility: Visibility,
        name: IdentifierAST,
        generic_parameters: &[GenericParameter],
        where_predicates: &[WherePredicate],
        methods: &[Function],
        inherits: Option<&[TypeConstructor]>,
        docstring: Option<&str>,
    ) {
        self.visit_generic_parameters(generic_parameters);
        self.visit_where_predicates(where_predicates);
        self.visit_methods(methods);
        self.visit_inherits(inherits);
    }

    fn visit_struct(
        &mut self,
        visibility: Visibility,
        name: IdentifierAST,
        generic_parameters: &[GenericParameter],
        where_predicates: &[WherePredicate],
        fields: &[StructField],
        methods: &[Function],
        implements: Option<&[TypeConstructor]>,
        docstring: Option<&str>,
    ) {
        self.visit_generic_parameters(generic_parameters);
        self.visit_where_predicates(where_predicates);
        self.visit_struct_fields(fields);
        self.visit_methods(methods);
        self.visit_implements(implements);
    }

    fn visit_tuple_like_struct(
        &mut self,
        visibility: Visibility,
        name: IdentifierAST,
        generic_parameters: &[GenericParameter],
        where_predicates: &[WherePredicate],
        fields: &[TupleField],
        methods: &[Function],
        implements: Option<&[TypeConstructor]>,
        docstring: Option<&str>,
    ) {
        self.visit_generic_parameters(generic_parameters);
        self.visit_where_predicates(where_predicates);
        self.visit_tuple_fields(fields);
        self.visit_methods(methods);
        self.visit_implements(implements);
    }

    fn visit_type_alias(&mut self, alias: &TypeAlias) {
        self.visit_generic_parameters(&alias.generic_parameters);
        self.visit_type(&alias.value);
    }

    fn visit_tuple_fields(&mut self, fields: &[TupleField]) {
        for field in fields {
            self.visit_tuple_field(field);
        }
    }

    fn visit_tuple_field(&mut self, field: &TupleField) {
        self.visit_type(&field.ty);
    }

    fn visit_struct_fields(&mut self, fields: &[StructField]) {
        for field in fields {
            self.visit_struct_field(field);
        }
    }

    fn visit_struct_field(&mut self, field: &StructField) {
        self.visit_type(&field.ty);
    }

    fn visit_generic_parameters(&mut self, generic_parameters: &[GenericParameter]) {
        for generic_parameter in generic_parameters {
            self.visit_generic_parameter(generic_parameter);
        }
    }

    fn visit_generic_parameter(&mut self, generic_parameter: &GenericParameter) {
        if let Some(default_value) = &generic_parameter.default_value {
            self.visit_type(default_value);
        }

        if let Some(bounds) = &generic_parameter.bounds {
            self.visit_bounds(bounds);
        }
    }

    fn visit_where_predicates(&mut self, predicates: &[WherePredicate]) {
        for predicate in predicates {
            self.visit_where_predicate(predicate);
        }
    }

    fn visit_where_predicate(&mut self, predicate: &WherePredicate) {
        self.visit_type(&predicate.ty);
        self.visit_bounds(&predicate.bounds);
    }

    fn visit_function(&mut self, function: &Function) {
        if let Some(body) = &function.body {
            self.visit_statements_block(body);
        }
    }

    fn visit_method(&mut self, method: &Function) {
        self.visit_function(method);
    }

    fn visit_methods(&mut self, methods: &[Function]) {
        for method in methods {
            self.visit_method(method);
        }
    }

    fn visit_implements(&mut self, implements: Option<&[TypeConstructor]>) {
        if let Some(implements) = implements {
            for interface in implements {
                self.visit_type_constructor(interface);
            }
        }
    }

    fn visit_inherits(&mut self, inherits: Option<&[TypeConstructor]>) {
        if let Some(inherits) = inherits {
            for interface in inherits {
                self.visit_type_constructor(interface);
            }
        }
    }

    fn visit_statements_block(&mut self, statements: &[Statement]) {
        for statement in statements {
            self.visit_statement(statement);
        }
    }

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

    fn visit_break_statement(&mut self, location: Location) {}

    fn visit_continue_statement(&mut self, location: Location) {}

    fn visit_defer_expression(&mut self, call: &Expression) {}

    fn visit_expression_statement(&mut self, expression: &Expression, has_semicolon: bool) {}

    fn visit_let_statement(&mut self, pattern: &Pattern, value: &Expression, ty: Option<&Type>) {
        self.visit_pattern(pattern);
        self.visit_expression(value);

        if let Some(ty) = ty {
            self.visit_type(ty);
        }
    }

    fn visit_return_statement(&mut self, expression: &Expression) {
        self.visit_expression(expression);
    }

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
        }
    }

    fn visit_grouped_pattern(&mut self, location: Location, inner: &Pattern) {
        self.visit_pattern(inner);
    }

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

    fn visit_list_pattern(&mut self, location: Location, inner_patterns: &[Pattern]) {
        for pattern in inner_patterns {
            self.visit_pattern(pattern);
        }
    }

    fn visit_literal_pattern(&mut self, literal: &Literal) {}

    fn visit_or_pattern(&mut self, left: &Pattern, right: &Pattern) {
        self.visit_pattern(left);
        self.visit_pattern(right);
    }

    fn visit_path_pattern(&mut self, path: &Path) {}

    fn visit_rest_pattern(&mut self, location: Location) {}

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

    fn visit_struct_field_pattern(&mut self, field: &StructFieldPattern) {}

    fn visit_tuple_pattern(&mut self, location: Location, elements: &[Pattern]) {}

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

    fn visit_type(&mut self, ty: &Type) {
        match ty {
            Type::Constructor(constructor) => self.visit_type_constructor(constructor),
            Type::Function {
                location,
                parameter_types,
                return_type,
            } => self.visit_function_type(*location, parameter_types, return_type),
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
        }
    }

    fn visit_type_arguments(&mut self, arguments: &[Type]) {
        for argument in arguments {
            self.visit_type(argument);
        }
    }

    fn visit_type_constructor(&mut self, constructor: &TypeConstructor) {
        self.visit_type_arguments(&constructor.arguments);
    }

    fn visit_function_type(
        &mut self,
        location: Location,
        parameter_types: &[Type],
        return_type: &Type,
    ) {
        for parameter_type in parameter_types {
            self.visit_type(parameter_type);
        }

        self.visit_type(return_type);
    }

    fn visit_interface_object_type(&mut self, location: Location, bounds: &[TypeConstructor]) {
        self.visit_bounds(bounds);
    }

    fn visit_bounds(&mut self, bounds: &[TypeConstructor]) {
        for bound in bounds {
            self.visit_type_constructor(bound);
        }
    }

    fn visit_parenthesized_type(&mut self, location: Location, inner: &Type) {
        self.visit_type(inner);
    }

    fn visit_tuple_type(&mut self, location: Location, element_types: &[Type]) {
        for element_type in element_types {
            self.visit_type(element_type);
        }
    }

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
        }
    }

    fn visit_as_expression(&mut self, location: Location, left: &Expression, right: &Type) {
        self.visit_expression(left);
        self.visit_type(right);
    }

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

    fn visit_field_access_expression(
        &mut self,
        location: Location,
        left: &Expression,
        right: IdentifierAST,
    ) {
        self.visit_expression(left);
    }

    fn visit_identifier_expression(&mut self, identifier: IdentifierAST) {}

    fn visit_list_expression(&mut self, location: Location, elements: &[Expression]) {
        for element in elements {
            self.visit_expression(element);
        }
    }

    fn visit_literal_expression(&mut self, literal: &Literal) {}

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

    fn visit_lambda_function_parameter(&mut self, parameter: &LambdaFunctionParameter) {
        if let Some(ty) = &parameter.ty {
            self.visit_type(ty);
        }
    }

    fn visit_loop_expression(&mut self, location: Location, statements_block: &[Statement]) {
        self.visit_statements_block(statements_block);
    }

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

    fn visit_match_expression_item(&mut self, item: &MatchExpressionItem) {
        self.visit_pattern(&item.left);
        self.visit_expression(&item.right);
    }

    fn visit_parenthesized_expression(&mut self, location: Location, inner: &Expression) {
        self.visit_expression(inner);
    }

    fn visit_postfix_expression(
        &mut self,
        location: Location,
        inner: &Expression,
        operator: PostfixOperator,
    ) {
        self.visit_expression(inner);
    }

    fn visit_prefix_expression(
        &mut self,
        location: Location,
        inner: &Expression,
        operator: PrefixOperator,
    ) {
        self.visit_expression(inner);
    }

    fn visit_statements_block_expression(&mut self, location: Location, block: &[Statement]) {
        self.visit_statements_block(block);
    }

    fn visit_struct_expression(
        &mut self,
        location: Location,
        left: &Expression,
        fields: &[StructFieldExpression],
    ) {
        self.visit_expression(left);
        self.visit_struct_field_expressions(fields);
    }

    fn visit_struct_field_expressions(&mut self, fields: &[StructFieldExpression]) {
        for field in fields {
            self.visit_struct_field_expression(field);
        }
    }

    fn visit_struct_field_expression(&mut self, field: &StructFieldExpression) {
        if let Some(value) = &field.value {
            self.visit_expression(value);
        }
    }

    fn visit_tuple_expression(&mut self, location: Location, elements: &[Expression]) {
        for element in elements {
            self.visit_expression(element);
        }
    }

    fn visit_while_expression(
        &mut self,
        location: Location,
        condition: &Expression,
        statements_block: &[Statement],
    ) {
        self.visit_expression(condition);

        self.visit_statements_block(statements_block);
    }

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
