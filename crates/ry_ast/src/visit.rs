//! AST walker. Each overridden visit method has full control over what
//! happens with its node, it can do its own traversal of the node's children,
//! call `visit::walk_*` to apply the default traversal algorithm, or prevent
//! deeper traversal by doing nothing.

use crate::{
    BinaryOperator, EnumItem, Expression, Function, FunctionParameter, FunctionSignature,
    GenericArgument, GenericParameter, IdentifierAst, Impl, ImportPath, LambdaFunctionParameter,
    Literal, MatchExpressionItem, Module, ModuleItem, NotSelfFunctionParameter, Path, Pattern,
    PostfixOperator, PrefixOperator, SelfFunctionParameter, Statement, StatementsBlock,
    StructExpressionItem, StructField, StructFieldPattern, TraitItem, TupleField, Type, TypeAlias,
    TypePath, TypePathSegment, Visibility, WherePredicate,
};

pub trait Visitor<'ast>: Sized {
    fn visit_identifier(&mut self, _identifier: IdentifierAst) {}

    fn visit_path(&mut self, path: &'ast Path) {
        walk_path(self, path);
    }

    fn visit_import_path(&mut self, _path: &'ast ImportPath) {}

    fn visit_module(&mut self, module: &'ast Module) {
        walk_module(self, module);
    }

    fn visit_module_docstring(&mut self, _docstring: Option<&'ast str>) {}

    fn visit_module_item(&mut self, item: &'ast ModuleItem) {
        walk_module_item(self, item);
    }

    fn visit_local_docstring(&mut self, _docstring: Option<&'ast str>) {}

    fn visit_type_implementation(&mut self, implementation: &'ast Impl) {
        walk_type_implementation(self, implementation);
    }

    fn visit_function(&mut self, function: &'ast Function) {
        walk_function(self, function);
    }

    fn visit_function_signature(&mut self, signature: &'ast FunctionSignature) {
        walk_function_signature(self, signature);
    }

    fn visit_visibility(&mut self, _visibility: Visibility) {}

    fn visit_where_predicates(&mut self, items: Option<&'ast [WherePredicate]>) {
        walk_where_predicates(self, items);
    }

    fn visit_where_predicate(&mut self, item: &'ast WherePredicate) {
        walk_there_predicate(self, item);
    }

    fn visit_enum_items(&mut self, items: &'ast [EnumItem]) {
        walk_enum_items(self, items);
    }

    fn visit_enum_item(&mut self, item: &'ast EnumItem) {
        walk_enum_item(self, item);
    }

    fn visit_struct_fields(&mut self, fields: &'ast [StructField]) {
        walk_struct_fields(self, fields);
    }

    fn visit_struct_field(&mut self, field: &'ast StructField) {
        walk_struct_field(self, field);
    }

    fn visit_generic_parameters(&mut self, parameters: Option<&'ast [GenericParameter]>) {
        walk_generic_parameters(self, parameters);
    }

    fn visit_generic_parameter(&mut self, parameter: &'ast GenericParameter) {
        walk_generic_parameter(self, parameter);
    }

    fn visit_trait_items(&mut self, items: &'ast [TraitItem]) {
        walk_trait_items(self, items);
    }

    fn visit_trait_item(&mut self, item: &'ast TraitItem) {
        walk_trait_item(self, item);
    }

    fn visit_type_alias(&mut self, alias: &'ast TypeAlias) {
        walk_type_alias(self, alias);
    }

    fn visit_tuple_fields(&mut self, fields: &'ast [TupleField]) {
        walk_tuple_fields(self, fields);
    }

    fn visit_tuple_field(&mut self, field: &'ast TupleField) {
        walk_tuple_field(self, field);
    }

    fn visit_type(&mut self, ty: &'ast Type) {
        walk_type(self, ty);
    }

    fn visit_type_path(&mut self, path: &'ast TypePath) {
        walk_type_path(self, path);
    }

    fn visit_type_path_segment(&mut self, segment: &'ast TypePathSegment) {
        walk_type_path_segment(self, segment);
    }

    fn visit_generic_arguments(&mut self, arguments: &'ast [GenericArgument]) {
        walk_generic_arguments(self, arguments);
    }

    fn visit_generic_argument(&mut self, argument: &'ast GenericArgument) {
        walk_generic_argument(self, argument);
    }

    fn visit_trait_bounds(&mut self, bounds: &'ast [TypePath]) {
        walk_trait_bounds(self, bounds);
    }

    fn visit_if_blocks(&mut self, blocks: &'ast [(Expression, StatementsBlock)]) {
        walk_if_blocks(self, blocks);
    }

    fn visit_if_block(&mut self, block: &'ast (Expression, StatementsBlock)) {
        walk_if_block(self, block);
    }

    fn visit_statements_block(&mut self, block: &'ast StatementsBlock) {
        walk_statements_block(self, block);
    }

    fn visit_statement(&mut self, statement: &'ast Statement) {
        walk_statement(self, statement);
    }

    fn visit_expression(&mut self, expression: &'ast Expression) {
        walk_expression(self, expression);
    }

    fn visit_lambda_function_parameters(&mut self, parameters: &'ast [LambdaFunctionParameter]) {
        walk_lambda_function_parameters(self, parameters);
    }

    fn visit_lambda_function_parameter(&mut self, parameter: &'ast LambdaFunctionParameter) {
        walk_lambda_function_parameter(self, parameter);
    }

    fn visit_function_parameters(&mut self, parameters: &'ast [FunctionParameter]) {
        walk_function_parameters(self, parameters);
    }

    fn visit_function_parameter(&mut self, parameter: &'ast FunctionParameter) {
        walk_function_parameter(self, parameter);
    }

    fn visit_self_function_parameter(&mut self, parameter: &'ast SelfFunctionParameter) {
        walk_self_function_parameter(self, parameter);
    }

    fn visit_not_self_function_parameter(&mut self, parameter: &'ast NotSelfFunctionParameter) {
        walk_not_self_function_parameter(self, parameter);
    }

    fn visit_match_expression_items(&mut self, items: &'ast [MatchExpressionItem]) {
        walk_match_expression_items(self, items);
    }

    fn visit_match_expression_item(&mut self, item: &'ast MatchExpressionItem) {
        walk_match_expression_item(self, item);
    }

    fn visit_struct_expression_items(&mut self, items: &'ast [StructExpressionItem]) {
        walk_struct_expression_items(self, items);
    }

    fn visit_struct_expression_item(&mut self, item: &'ast StructExpressionItem) {
        walk_struct_expression_item(self, item);
    }

    fn visit_binary_operator(&mut self, _operator: BinaryOperator) {}

    fn visit_postfix_operator(&mut self, _operator: PostfixOperator) {}

    fn visit_prefix_operator(&mut self, _operator: PrefixOperator) {}

    fn visit_pattern(&mut self, pattern: &'ast Pattern) {
        walk_pattern(self, pattern);
    }

    fn visit_literal(&mut self, _literal: &'ast Literal) {}

    fn visit_struct_field_patterns(&mut self, patterns: &'ast [StructFieldPattern]) {
        walk_struct_field_patterns(self, patterns);
    }

    fn visit_struct_field_pattern(&mut self, pattern: &'ast StructFieldPattern) {
        walk_struct_field_pattern(self, pattern);
    }
}

#[macro_export]
macro_rules! walk_list {
    ($visitor:expr, $method:ident, $list:expr$(, $($extra_args:expr),* )?) => {
        #[allow(for_loops_over_fallibles)]
        for elem in $list {
            $visitor.$method(elem $(, $($extra_args,)* )?)
        }
    };
}

pub fn walk_path<'ast, V>(visitor: &mut V, path: &'ast Path)
where
    V: Visitor<'ast>,
{
    for identifier in &path.identifiers {
        visitor.visit_identifier(*identifier);
    }
}

pub fn walk_module<'ast, V>(visitor: &mut V, module: &'ast Module)
where
    V: Visitor<'ast>,
{
    visitor.visit_module_docstring(module.docstring.as_deref());

    for item in &module.items {
        visitor.visit_module_item(item);
    }
}

pub fn walk_module_item<'ast, V>(visitor: &mut V, item: &'ast ModuleItem)
where
    V: Visitor<'ast>,
{
    match item {
        ModuleItem::Enum {
            visibility,
            name,
            generic_parameters,
            where_predicates,
            items,
            docstring,
        } => {
            visitor.visit_local_docstring(docstring.as_deref());
            visitor.visit_visibility(*visibility);
            visitor.visit_identifier(*name);
            visitor.visit_generic_parameters(generic_parameters.as_deref());
            visitor.visit_where_predicates(where_predicates.as_deref());
            visitor.visit_enum_items(items);
        }
        ModuleItem::Function(function) => {
            visitor.visit_function(function);
        }
        ModuleItem::Import { path, .. } => {
            visitor.visit_import_path(path);
        }
        ModuleItem::Trait {
            visibility,
            name,
            generic_parameters,
            where_predicates,
            items,
            docstring,
        } => {
            visitor.visit_local_docstring(docstring.as_deref());
            visitor.visit_visibility(*visibility);
            visitor.visit_identifier(*name);
            visitor.visit_generic_parameters(generic_parameters.as_deref());
            visitor.visit_where_predicates(where_predicates.as_deref());
            visitor.visit_trait_items(items);
        }
        ModuleItem::TupleLikeStruct {
            visibility,
            name,
            generic_parameters,
            where_predicates,
            fields,
            docstring,
        } => {
            visitor.visit_local_docstring(docstring.as_deref());
            visitor.visit_visibility(*visibility);
            visitor.visit_identifier(*name);
            visitor.visit_generic_parameters(generic_parameters.as_deref());
            visitor.visit_where_predicates(where_predicates.as_deref());
            visitor.visit_tuple_fields(fields);
        }
        ModuleItem::Impl(implementation) => visitor.visit_type_implementation(implementation),
        ModuleItem::Struct {
            visibility,
            name,
            generic_parameters,
            where_predicates,
            fields,
            docstring,
        } => {
            visitor.visit_visibility(*visibility);
            visitor.visit_identifier(*name);
            visitor.visit_generic_parameters(generic_parameters.as_deref());
            visitor.visit_where_predicates(where_predicates.as_deref());
            visitor.visit_struct_fields(fields);
            visitor.visit_local_docstring(docstring.as_deref());
        }
        ModuleItem::TypeAlias(alias) => visitor.visit_type_alias(alias),
    }
}

pub fn walk_type_implementation<'ast, V>(visitor: &mut V, implementation: &'ast Impl)
where
    V: Visitor<'ast>,
{
    visitor.visit_local_docstring(implementation.docstring.as_deref());

    if let Some(r#trait) = &implementation.r#trait {
        visitor.visit_type(r#trait);
    }

    visitor.visit_type(&implementation.ty);
    visitor.visit_where_predicates(implementation.where_predicates.as_deref());

    visitor.visit_trait_items(&implementation.items);
}

pub fn walk_function<'ast, V>(visitor: &mut V, function: &'ast Function)
where
    V: Visitor<'ast>,
{
    visitor.visit_function_signature(&function.signature);

    if let Some(body) = &function.body {
        visitor.visit_statements_block(body);
    }
}

pub fn walk_function_signature<'ast, V>(visitor: &mut V, signature: &'ast FunctionSignature)
where
    V: Visitor<'ast>,
{
    visitor.visit_visibility(signature.visibility);
    visitor.visit_identifier(signature.name);
    visitor.visit_generic_parameters(signature.generic_parameters.as_deref());
    visitor.visit_function_parameters(&signature.parameters);
}

pub fn walk_where_predicates<'ast, V>(visitor: &mut V, items: Option<&'ast [WherePredicate]>)
where
    V: Visitor<'ast>,
{
    if let Some(items) = items {
        for item in items {
            visitor.visit_where_predicate(item);
        }
    }
}

pub fn walk_there_predicate<'ast, V>(visitor: &mut V, item: &'ast WherePredicate)
where
    V: Visitor<'ast>,
{
    match item {
        WherePredicate::Eq { left, right } => {
            visitor.visit_type(left);
            visitor.visit_type(right);
        }
        WherePredicate::Satisfies { ty, bounds } => {
            visitor.visit_type(ty);
            visitor.visit_trait_bounds(bounds);
        }
    }
}

pub fn walk_enum_items<'ast, V>(visitor: &mut V, items: &'ast [EnumItem])
where
    V: Visitor<'ast>,
{
    for item in items {
        visitor.visit_enum_item(item);
    }
}

pub fn walk_enum_item<'ast, V>(visitor: &mut V, item: &'ast EnumItem)
where
    V: Visitor<'ast>,
{
    match item {
        EnumItem::Just { name, docstring } => {
            visitor.visit_local_docstring(docstring.as_deref());
            visitor.visit_identifier(*name);
        }
        EnumItem::Struct {
            name,
            fields,
            docstring,
        } => {
            visitor.visit_local_docstring(docstring.as_deref());
            visitor.visit_identifier(*name);
            visitor.visit_struct_fields(fields);
        }
        EnumItem::TupleLike {
            name,
            fields,
            docstring,
        } => {
            visitor.visit_local_docstring(docstring.as_deref());
            visitor.visit_identifier(*name);
            visitor.visit_tuple_fields(fields);
        }
    }
}

pub fn walk_struct_fields<'ast, V>(visitor: &mut V, fields: &'ast [StructField])
where
    V: Visitor<'ast>,
{
    for field in fields {
        visitor.visit_struct_field(field);
    }
}

pub fn walk_struct_field<'ast, V>(visitor: &mut V, field: &'ast StructField)
where
    V: Visitor<'ast>,
{
    visitor.visit_visibility(field.visibility);
    visitor.visit_identifier(field.name);
    visitor.visit_type(&field.ty);
}

pub fn walk_trait_items<'ast, V>(visitor: &mut V, items: &'ast [TraitItem])
where
    V: Visitor<'ast>,
{
    for item in items {
        visitor.visit_trait_item(item);
    }
}

pub fn walk_trait_item<'ast, V>(visitor: &mut V, item: &'ast TraitItem)
where
    V: Visitor<'ast>,
{
    match item {
        TraitItem::TypeAlias(alias) => visitor.visit_type_alias(alias),
        TraitItem::AssociatedFunction(function) => visitor.visit_function(function),
    }
}

pub fn walk_type_alias<'ast, V>(visitor: &mut V, alias: &'ast TypeAlias)
where
    V: Visitor<'ast>,
{
    visitor.visit_visibility(alias.visibility);
    visitor.visit_identifier(alias.name);
    visitor.visit_generic_parameters(alias.generic_parameters.as_deref());

    if let Some(bounds) = &alias.bounds {
        visitor.visit_trait_bounds(bounds);
    }

    if let Some(value) = &alias.value {
        visitor.visit_type(value);
    }
}

pub fn walk_tuple_fields<'ast, V>(visitor: &mut V, fields: &'ast [TupleField])
where
    V: Visitor<'ast>,
{
    for field in fields {
        visitor.visit_tuple_field(field);
    }
}

pub fn walk_tuple_field<'ast, V>(visitor: &mut V, field: &'ast TupleField)
where
    V: Visitor<'ast>,
{
    visitor.visit_visibility(field.visibility);
    visitor.visit_type(&field.ty);
}

pub fn walk_generic_parameters<'ast, V>(
    visitor: &mut V,
    parameters: Option<&'ast [GenericParameter]>,
) where
    V: Visitor<'ast>,
{
    if let Some(parameters) = parameters {
        walk_list!(visitor, visit_generic_parameter, parameters);
    }
}

pub fn walk_generic_parameter<'ast, V>(visitor: &mut V, parameter: &'ast GenericParameter)
where
    V: Visitor<'ast>,
{
    visitor.visit_identifier(parameter.name);

    if let Some(bounds) = &parameter.bounds {
        visitor.visit_trait_bounds(bounds);
    }

    if let Some(default_value) = &parameter.default_value {
        visitor.visit_type(default_value);
    }
}

pub fn walk_type<'ast, V>(visitor: &mut V, ty: &'ast Type)
where
    V: Visitor<'ast>,
{
    match ty {
        Type::Path(path) => visitor.visit_type_path(path),
        Type::Tuple { element_types, .. } => {
            walk_list!(visitor, visit_type, element_types);
        }
        Type::Function {
            parameter_types,
            return_type,
            ..
        } => {
            walk_list!(visitor, visit_type, parameter_types);

            visitor.visit_type(return_type);
        }
        Type::Parenthesized { inner, .. } => {
            visitor.visit_type(inner);
        }
        Type::TraitObject { bounds, .. } => {
            visitor.visit_trait_bounds(bounds);
        }
        Type::WithQualifiedPath {
            left,
            right,
            segments,
            ..
        } => {
            visitor.visit_type(left);
            visitor.visit_type_path(right);

            walk_list!(visitor, visit_type_path_segment, segments);
        }
    }
}

pub fn walk_type_path<'ast, V>(visitor: &mut V, path: &'ast TypePath)
where
    V: Visitor<'ast>,
{
    walk_list!(visitor, visit_type_path_segment, &path.segments);
}

pub fn walk_type_path_segment<'ast, V>(visitor: &mut V, segment: &'ast TypePathSegment)
where
    V: Visitor<'ast>,
{
    visitor.visit_path(&segment.path);

    if let Some(generic_arguments) = &segment.generic_arguments {
        visitor.visit_generic_arguments(generic_arguments);
    }
}

pub fn walk_generic_arguments<'ast, V>(visitor: &mut V, arguments: &'ast [GenericArgument])
where
    V: Visitor<'ast>,
{
    walk_list!(visitor, visit_generic_argument, arguments);
}

pub fn walk_generic_argument<'ast, V>(visitor: &mut V, argument: &'ast GenericArgument)
where
    V: Visitor<'ast>,
{
    match argument {
        GenericArgument::Type(ty) => visitor.visit_type(ty),
        GenericArgument::AssociatedType { name, value } => {
            visitor.visit_identifier(*name);
            visitor.visit_type(value);
        }
    }
}

pub fn walk_trait_bounds<'ast, V>(visitor: &mut V, bounds: &'ast [TypePath])
where
    V: Visitor<'ast>,
{
    for bound in bounds {
        visitor.visit_type_path(bound);
    }
}

pub fn walk_if_blocks<'ast, V>(visitor: &mut V, blocks: &'ast [(Expression, StatementsBlock)])
where
    V: Visitor<'ast>,
{
    for block in blocks {
        visitor.visit_if_block(block);
    }
}

pub fn walk_if_block<'ast, V>(visitor: &mut V, block: &'ast (Expression, StatementsBlock))
where
    V: Visitor<'ast>,
{
    visitor.visit_expression(&block.0);
    visitor.visit_statements_block(&block.1);
}

pub fn walk_statements_block<'ast, V>(visitor: &mut V, block: &'ast StatementsBlock)
where
    V: Visitor<'ast>,
{
    for statement in block {
        visitor.visit_statement(statement);
    }
}

pub fn walk_statement<'ast, V>(visitor: &mut V, statement: &'ast Statement)
where
    V: Visitor<'ast>,
{
    match statement {
        Statement::Defer { call } => {
            visitor.visit_expression(call);
        }
        Statement::Expression { expression, .. } | Statement::Return { expression } => {
            visitor.visit_expression(expression);
        }
        Statement::Let { pattern, value, ty } => {
            visitor.visit_pattern(pattern);

            visitor.visit_expression(value);

            if let Some(ty) = ty {
                visitor.visit_type(ty);
            }
        }
        Statement::Break { .. } | Statement::Continue { .. } => {}
    }
}

pub fn walk_expression<'ast, V>(visitor: &mut V, expression: &'ast Expression)
where
    V: Visitor<'ast>,
{
    match expression {
        Expression::As { left, right, .. } => {
            visitor.visit_expression(left);
            visitor.visit_type(right);
        }
        Expression::Binary {
            left,
            operator,
            right,
            ..
        } => {
            visitor.visit_expression(left);
            visitor.visit_binary_operator(*operator);
            visitor.visit_expression(right);
        }
        Expression::Call {
            left, arguments, ..
        } => {
            visitor.visit_expression(left);
            walk_list!(visitor, visit_expression, arguments);
        }
        Expression::FieldAccess { left, right, .. } => {
            visitor.visit_expression(left);
            visitor.visit_identifier(*right);
        }
        Expression::Lambda {
            parameters,
            return_type,
            block,
            ..
        } => {
            visitor.visit_lambda_function_parameters(parameters);

            if let Some(return_type) = return_type {
                visitor.visit_type(return_type);
            }

            visitor.visit_statements_block(block);
        }
        Expression::GenericArguments {
            left,
            generic_arguments,
            ..
        } => {
            visitor.visit_expression(left);

            for argument in generic_arguments {
                visitor.visit_generic_argument(argument);
            }
        }
        Expression::Identifier(identifier) => visitor.visit_identifier(*identifier),
        Expression::If {
            if_blocks, r#else, ..
        } => {
            visitor.visit_if_blocks(if_blocks);

            if let Some(r#else) = r#else {
                visitor.visit_statements_block(r#else);
            }
        }
        Expression::List { elements, .. } | Expression::Tuple { elements, .. } => {
            walk_list!(visitor, visit_expression, elements);
        }
        Expression::Literal(literal) => visitor.visit_literal(literal),
        Expression::Match {
            expression, block, ..
        } => {
            visitor.visit_expression(expression);
            visitor.visit_match_expression_items(block);
        }
        Expression::Parenthesized { inner, .. } => {
            visitor.visit_expression(inner);
        }
        Expression::Postfix {
            inner, operator, ..
        } => {
            visitor.visit_expression(inner);
            visitor.visit_postfix_operator(*operator);
        }
        Expression::Prefix {
            inner, operator, ..
        } => {
            visitor.visit_expression(inner);
            visitor.visit_prefix_operator(*operator);
        }
        Expression::StatementsBlock { block, .. } => {
            visitor.visit_statements_block(block);
        }
        Expression::Struct { left, fields, .. } => {
            visitor.visit_expression(left);
            visitor.visit_struct_expression_items(fields);
        }
        Expression::While {
            condition, body, ..
        } => {
            visitor.visit_expression(condition);
            visitor.visit_statements_block(body);
        }
    }
}

pub fn walk_lambda_function_parameter<'ast, V>(
    visitor: &mut V,
    parameter: &'ast LambdaFunctionParameter,
) where
    V: Visitor<'ast>,
{
    visitor.visit_identifier(parameter.name);

    if let Some(ty) = &parameter.ty {
        visitor.visit_type(ty);
    }
}

pub fn walk_lambda_function_parameters<'ast, V>(
    visitor: &mut V,
    parameters: &'ast [LambdaFunctionParameter],
) where
    V: Visitor<'ast>,
{
    walk_list!(visitor, visit_lambda_function_parameter, parameters);
}

pub fn walk_function_parameters<'ast, V>(visitor: &mut V, parameters: &'ast [FunctionParameter])
where
    V: Visitor<'ast>,
{
    walk_list!(visitor, visit_function_parameter, parameters);
}

pub fn walk_function_parameter<'ast, V>(visitor: &mut V, parameter: &'ast FunctionParameter)
where
    V: Visitor<'ast>,
{
    match parameter {
        FunctionParameter::NotSelfParameter(parameter) => {
            visitor.visit_not_self_function_parameter(parameter)
        }
        FunctionParameter::SelfParameter(parameter) => {
            visitor.visit_self_function_parameter(parameter)
        }
    }
}

pub fn walk_self_function_parameter<'ast, V>(
    visitor: &mut V,
    parameter: &'ast SelfFunctionParameter,
) where
    V: Visitor<'ast>,
{
    if let Some(ty) = &parameter.ty {
        visitor.visit_type(ty);
    }
}

pub fn walk_not_self_function_parameter<'ast, V>(
    visitor: &mut V,
    parameter: &'ast NotSelfFunctionParameter,
) where
    V: Visitor<'ast>,
{
    visitor.visit_identifier(parameter.name);
    // visitor.visit_type(&parameter.ty);
}

pub fn walk_match_expression_item<'ast, V>(visitor: &mut V, item: &'ast MatchExpressionItem)
where
    V: Visitor<'ast>,
{
    visitor.visit_pattern(&item.left);
    visitor.visit_expression(&item.right);
}

pub fn walk_match_expression_items<'ast, V>(visitor: &mut V, items: &'ast [MatchExpressionItem])
where
    V: Visitor<'ast>,
{
    walk_list!(visitor, visit_match_expression_item, items);
}

pub fn walk_struct_expression_item<'ast, V>(visitor: &mut V, item: &'ast StructExpressionItem)
where
    V: Visitor<'ast>,
{
    visitor.visit_identifier(item.name);

    if let Some(value) = &item.value {
        visitor.visit_expression(value);
    }
}

pub fn walk_struct_expression_items<'ast, V>(visitor: &mut V, items: &'ast [StructExpressionItem])
where
    V: Visitor<'ast>,
{
    walk_list!(visitor, visit_struct_expression_item, items);
}

pub fn walk_pattern<'ast, V>(visitor: &mut V, pattern: &'ast Pattern)
where
    V: Visitor<'ast>,
{
    match pattern {
        Pattern::Grouped { inner, .. } => {
            visitor.visit_pattern(inner);
        }
        Pattern::Identifier {
            identifier,
            pattern,
            ..
        } => {
            visitor.visit_identifier(*identifier);

            if let Some(pattern) = pattern {
                visitor.visit_pattern(pattern);
            }
        }
        Pattern::List { inner_patterns, .. } => {
            walk_list!(visitor, visit_pattern, inner_patterns);
        }
        Pattern::Literal(literal) => visitor.visit_literal(literal),
        Pattern::Or { left, right, .. } => {
            visitor.visit_pattern(left);
            visitor.visit_pattern(right);
        }
        Pattern::Path { path, .. } => {
            visitor.visit_path(path);
        }
        Pattern::Rest { .. } => {}

        Pattern::Struct { path, fields, .. } => {
            visitor.visit_path(path);
            visitor.visit_struct_field_patterns(fields);
        }
        Pattern::Tuple { elements, .. } => {
            walk_list!(visitor, visit_pattern, elements);
        }
        Pattern::TupleLike {
            path,
            inner_patterns,
            ..
        } => {
            visitor.visit_path(path);
            walk_list!(visitor, visit_pattern, inner_patterns);
        }
    }
}

pub fn walk_struct_field_patterns<'ast, V>(visitor: &mut V, patterns: &'ast [StructFieldPattern])
where
    V: Visitor<'ast>,
{
    walk_list!(visitor, visit_struct_field_pattern, patterns);
}

pub fn walk_struct_field_pattern<'ast, V>(visitor: &mut V, pattern: &'ast StructFieldPattern)
where
    V: Visitor<'ast>,
{
    match pattern {
        StructFieldPattern::NotRest {
            field_name,
            value_pattern,
            ..
        } => {
            visitor.visit_identifier(*field_name);

            if let Some(value_pattern) = value_pattern {
                visitor.visit_pattern(value_pattern);
            }
        }
        StructFieldPattern::Rest { .. } => {}
    }
}
