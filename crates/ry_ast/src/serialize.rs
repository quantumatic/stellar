//! Defines [`Serializer`] to serialize AST into a string.

use ry_interner::IdentifierInterner;

use crate::{
    visit::{
        walk_enum_items, walk_expression, walk_function, walk_generic_argument,
        walk_generic_arguments, walk_generic_parameter, walk_generic_parameters, walk_if_block,
        walk_if_blocks, walk_lambda_function_parameter, walk_lambda_function_parameters,
        walk_match_expression_item, walk_match_expression_items, walk_module, walk_module_item,
        walk_path, walk_statement, walk_statements_block, walk_struct_expression_item,
        walk_struct_expression_items, walk_struct_field, walk_struct_field_pattern,
        walk_struct_field_patterns, walk_struct_fields, walk_there_predicate, walk_trait_bounds,
        walk_trait_item, walk_trait_items, walk_tuple_field, walk_tuple_fields, walk_type,
        walk_type_alias, walk_type_implementation, walk_type_path, walk_type_path_segment,
        walk_where_predicates, Visitor,
    },
    BinaryOperator, EnumItem, Expression, Function, GenericArgument, GenericParameter,
    IdentifierAST, Impl, ImportPath, LambdaFunctionParameter, Literal, MatchExpressionItem, Module,
    ModuleItem, Path, Pattern, PostfixOperator, PrefixOperator, Statement, StatementsBlock,
    StructExpressionItem, StructField, StructFieldPattern, TraitItem, TupleField, Type, TypeAlias,
    TypePath, TypePathSegment, Visibility, WherePredicate,
};

/// A struct that allows to serialize a Ry module into a string, for debug purposes.
#[derive(Debug)]
pub struct Serializer<'i> {
    /// An interner used to resolve symbols in an AST.
    identifier_interner: &'i IdentifierInterner,

    /// Current indentation level
    identation: usize,

    /// An output string produced,
    output: String,
}

impl<'i> Serializer<'i> {
    /// Creates a new serializer instance.
    #[inline]
    #[must_use]
    pub const fn new(identifier_interner: &'i IdentifierInterner) -> Self {
        Self {
            identifier_interner,
            identation: 0,
            output: String::new(),
        }
    }

    /// Increments the current indentation level.
    #[inline]
    pub fn increment_indentation(&mut self) {
        self.identation += 1;
    }

    /// Decrements the current indentation level.
    #[inline]
    pub fn decrement_indentation(&mut self) {
        self.identation -= 1;
    }

    /// Pushes a string into the output.
    pub fn write<S>(&mut self, str: S)
    where
        S: AsRef<str>,
    {
        self.output.push_str(str.as_ref());
    }

    /// Pushes a newline into the output.
    #[inline]
    pub fn write_newline(&mut self) {
        self.output.push('\n');
    }

    /// Adds indentation symbols into the output.
    pub fn write_identation(&mut self) {
        for _ in 0..self.identation {
            self.write("\t");
        }
    }

    /// Returns the output string produced.
    #[inline]
    #[must_use]
    pub fn output(&self) -> &str {
        &self.output
    }

    /// Returns the owned output string produced.
    #[inline]
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // false-positive clippy lint
    pub fn take_output(self) -> String {
        self.output
    }
}

impl Visitor<'_> for Serializer<'_> {
    fn visit_binary_operator(&mut self, operator: BinaryOperator) {
        self.increment_indentation();
        self.write_identation();

        self.write(format!("BINARY_OP {}@{}", operator.raw, operator.location));

        self.decrement_indentation();
    }

    fn visit_enum_items(&mut self, items: &'_ [EnumItem]) {
        self.increment_indentation();
        self.write_identation();

        self.write("ENUM_ITEMS");
        self.write_newline();

        walk_enum_items(self, items);

        self.decrement_indentation();
    }

    fn visit_expression(&mut self, expression: &'_ Expression) {
        self.increment_indentation();
        self.write_identation();

        match expression {
            Expression::As { .. } => self.write("AS"),
            Expression::Binary { .. } => self.write("BINARY"),
            Expression::Call { .. } => self.write("CALL"),
            Expression::FieldAccess { .. } => self.write("FIELD_ACCESS"),
            Expression::GenericArguments { .. } => self.write("GENERIC_AGRUMENTS"),
            Expression::Identifier(..) => self.write("IDENTIFIER"),
            Expression::If { .. } => self.write("IF"),
            Expression::Lambda { .. } => self.write("LAMBDA"),
            Expression::List { .. } => self.write("LIST"),
            Expression::Literal(..) => self.write("LITERAL"),
            Expression::Loop { .. } => self.write("LOOP"),
            Expression::Match { .. } => self.write("MATCH"),
            Expression::Parenthesized { .. } => self.write("PARENTHESIZED"),
            Expression::Postfix { .. } => self.write("POSTFIX"),
            Expression::Prefix { .. } => self.write("PREFIX"),
            Expression::StatementsBlock { .. } => self.write("STATEMENTS_BLOCK"),
            Expression::Struct { .. } => self.write("STRUCT"),
            Expression::Tuple { .. } => self.write("TUPLE"),
            Expression::While { .. } => self.write("WHILE"),
        }

        self.write(format!(" <{}>", expression.location()));
        self.write_newline();
        walk_expression(self, expression);

        self.decrement_indentation();
    }

    fn visit_function(&mut self, function: &'_ Function) {
        self.increment_indentation();
        self.write_identation();

        self.write("FUNCTION");
        self.write_newline();
        walk_function(self, function);

        self.decrement_indentation();
    }

    fn visit_generic_argument(&mut self, argument: &'_ GenericArgument) {
        self.increment_indentation();
        self.write_identation();

        self.write("GENERIC_ARGUMENT");
        self.write_newline();
        walk_generic_argument(self, argument);

        self.decrement_indentation();
    }

    fn visit_generic_arguments(&mut self, arguments: &'_ [GenericArgument]) {
        self.increment_indentation();
        self.write_identation();

        self.write("GENERIC_ARGUMENTS");
        self.write_newline();
        walk_generic_arguments(self, arguments);

        self.decrement_indentation();
    }

    fn visit_generic_parameter(&mut self, parameter: &'_ GenericParameter) {
        self.increment_indentation();
        self.write_identation();

        self.write("GENERIC_PARAMETER");
        self.write_newline();
        walk_generic_parameter(self, parameter);

        self.decrement_indentation();
    }

    fn visit_generic_parameters(&mut self, parameters: Option<&'_ [GenericParameter]>) {
        self.increment_indentation();
        self.write_identation();

        self.write("GENERIC_PARAMETERS");
        self.write_newline();
        walk_generic_parameters(self, parameters);

        self.decrement_indentation();
    }

    fn visit_identifier(&mut self, identifier: IdentifierAST) {
        self.increment_indentation();
        self.write_identation();

        self.write(format!(
            "IDENTIFIER: {} <{}>",
            self.identifier_interner
                .resolve(identifier.symbol)
                .unwrap_or("?"),
            identifier.location
        ));
        self.write_newline();

        self.decrement_indentation();
    }

    fn visit_if_block(&mut self, block: &'_ (Expression, StatementsBlock)) {
        self.increment_indentation();
        self.write_identation();

        self.write("IF_BLOCK");
        self.write_newline();
        walk_if_block(self, block);

        self.decrement_indentation();
    }

    fn visit_if_blocks(&mut self, blocks: &'_ [(Expression, StatementsBlock)]) {
        self.increment_indentation();
        self.write_identation();

        self.write("IF_BLOCKS");
        self.write_newline();
        walk_if_blocks(self, blocks);

        self.decrement_indentation();
    }

    fn visit_import_path(&mut self, path: &'_ ImportPath) {
        self.increment_indentation();
        self.write_identation();

        self.write("IMPORT_PATH");
        self.write_newline();
        walk_path(self, &path.path);

        if let Some(r#as) = path.r#as {
            self.increment_indentation();
            self.write_identation();
            self.write("AS");
            self.write_newline();
            self.decrement_indentation();
            self.visit_identifier(r#as);
        }

        self.decrement_indentation();
    }

    fn visit_module_item(&mut self, item: &'_ ModuleItem) {
        self.increment_indentation();
        self.write_identation();

        match item {
            ModuleItem::Enum { .. } => self.write("ENUM_GLOBAL_ITEM"),
            ModuleItem::Function(..) => self.write("FUNCTION_GLOBAL_ITEM"),
            ModuleItem::Impl(..) => self.write("IMPL_GLOBAL_ITEM"),
            ModuleItem::Import { .. } => self.write("IMPORT"),
            ModuleItem::Struct { .. } => self.write("STRUCT_GLOBAL_ITEM"),
            ModuleItem::Trait { .. } => self.write("TRAIT_GLOBAL_ITEM"),
            ModuleItem::TupleLikeStruct { .. } => self.write("TUPLE_LIKE_STRUCT_GLOBAL_ITEM"),
            ModuleItem::TypeAlias(..) => self.write("TYPE_ALIAS_GLOBAL_ITEM"),
        }

        self.write_newline();
        walk_module_item(self, item);

        self.decrement_indentation();
    }

    fn visit_lambda_function_parameter(&mut self, parameter: &'_ LambdaFunctionParameter) {
        self.increment_indentation();
        self.write_identation();

        self.write("LAMBDA_FUNCTION_PARAMETER");
        self.write_newline();

        walk_lambda_function_parameter(self, parameter);

        self.decrement_indentation();
    }

    fn visit_lambda_function_parameters(&mut self, parameters: &'_ [LambdaFunctionParameter]) {
        self.increment_indentation();
        self.write_identation();

        self.write("LAMBDA_FUNCTION_PARAMETERS");
        self.write_newline();

        walk_lambda_function_parameters(self, parameters);

        self.decrement_indentation();
    }

    fn visit_literal(&mut self, literal: &'_ Literal) {
        self.increment_indentation();
        self.write_identation();

        self.write("LITERAL ");

        match literal {
            Literal::Boolean { value, .. } => self.write(format!("{value}")),
            Literal::Character { value, .. } => self.write(format!("'{value}'")),
            Literal::Float { value, .. } => self.write(format!("{value}")),
            Literal::Integer { value, .. } => self.write(format!("{value}")),
            Literal::String { value, .. } => self.write(format!("\"{value}\"")),
        }

        self.write(format!(" <{}>", literal.location()));
        self.write_newline();

        self.decrement_indentation();
    }

    fn visit_match_expression_item(&mut self, item: &'_ MatchExpressionItem) {
        self.increment_indentation();
        self.write_identation();

        self.write("MATCH_EXPRESSION_ITEM");
        self.write_newline();
        walk_match_expression_item(self, item);

        self.decrement_indentation();
    }

    fn visit_match_expression_items(&mut self, items: &'_ [MatchExpressionItem]) {
        self.increment_indentation();
        self.write_identation();

        self.write("MATCH_EXPRESSION_ITEMS");
        self.write_newline();
        walk_match_expression_items(self, items);

        self.decrement_indentation();
    }

    fn visit_module(&mut self, module: &'_ Module) {
        self.write("MODULE");
        self.write_newline();
        walk_module(self, module);
    }

    fn visit_path(&mut self, path: &'_ Path) {
        self.increment_indentation();
        self.write_identation();

        self.write(format!("PATH <{}>", path.location));
        self.write_newline();
        walk_path(self, path);

        self.decrement_indentation();
    }

    fn visit_pattern(&mut self, pattern: &'_ Pattern) {
        self.increment_indentation();
        self.write_identation();

        match pattern {
            Pattern::Grouped { .. } => self.write("GROUPED_PATTERN"),
            Pattern::Identifier { .. } => self.write("IDENTIFIER_PATTERN"),
            Pattern::List { .. } => self.write("LIST_PATTERN"),
            Pattern::Literal(..) => self.write("LITERAL_PATTERN"),
            Pattern::Or { .. } => self.write("OR_PATTERN"),
            Pattern::Path { .. } => self.write("PATH_PATTERN"),
            Pattern::Rest { .. } => self.write("REST_PATTERN"),
            Pattern::Struct { .. } => self.write("STRUCT_PATTERN"),
            Pattern::Tuple { .. } => self.write("TUPLE_PATTERN"),
            Pattern::TupleLike { .. } => self.write("TUPLE_LIKE_PATTERN"),
        }

        self.write(format!(" <{}>", pattern.location()));
        self.write_newline();

        self.decrement_indentation();
    }

    fn visit_postfix_operator(&mut self, operator: PostfixOperator) {
        self.increment_indentation();
        self.write_identation();

        self.write(format!("POSTFIX_OP {}@{}", operator.raw, operator.location));
        self.write_newline();

        self.decrement_indentation();
    }

    fn visit_prefix_operator(&mut self, operator: PrefixOperator) {
        self.increment_indentation();
        self.write_identation();

        self.write(format!("PREFIX_OP {}@{}", operator.raw, operator.location));
        self.write_newline();

        self.decrement_indentation();
    }

    fn visit_statement(&mut self, statement: &'_ Statement) {
        self.increment_indentation();
        self.write_identation();

        match statement {
            Statement::Break { .. } => self.write("BREAK_STATEMENT"),
            Statement::Continue { .. } => self.write("CONTINUE_STATEMENT"),
            Statement::Defer { .. } => self.write("DEFER_STATEMENT"),
            Statement::Expression { .. } => self.write("EXPRESSION_STATEMENT"),
            Statement::Let { .. } => self.write("LET_STATEMENT"),
            Statement::Return { .. } => self.write("RETURN_STATEMENT"),
        }

        self.write_newline();
        walk_statement(self, statement);

        self.decrement_indentation();
    }

    fn visit_statements_block(&mut self, block: &'_ StatementsBlock) {
        self.increment_indentation();
        self.write_identation();

        self.write("STATEMENTS_BLOCK");
        self.write_newline();

        walk_statements_block(self, block);

        self.decrement_indentation();
    }

    fn visit_struct_expression_item(&mut self, item: &'_ StructExpressionItem) {
        self.increment_indentation();
        self.write_identation();

        self.write("STRUCT_EXPRESSION_ITEM");
        self.write_newline();

        walk_struct_expression_item(self, item);

        self.decrement_indentation();
    }

    fn visit_struct_expression_items(&mut self, items: &'_ [StructExpressionItem]) {
        self.increment_indentation();
        self.write_identation();

        self.write("STRUCT_EXPRESSION_ITEMS");
        self.write_newline();

        walk_struct_expression_items(self, items);

        self.decrement_indentation();
    }

    fn visit_struct_field(&mut self, field: &'_ StructField) {
        self.increment_indentation();
        self.write_identation();

        self.write("STRUCT_FIELD");
        self.write_newline();

        walk_struct_field(self, field);

        self.decrement_indentation();
    }

    fn visit_struct_field_pattern(&mut self, pattern: &'_ StructFieldPattern) {
        self.increment_indentation();
        self.write_identation();

        self.write("STRUCT_FIELD_PATTERN");
        self.write_newline();

        walk_struct_field_pattern(self, pattern);

        self.decrement_indentation();
    }

    fn visit_struct_field_patterns(&mut self, patterns: &'_ [StructFieldPattern]) {
        self.increment_indentation();
        self.write_identation();

        self.write("STRUCT_FIELD_PATTERNS");
        self.write_newline();

        walk_struct_field_patterns(self, patterns);

        self.decrement_indentation();
    }

    fn visit_struct_fields(&mut self, fields: &'_ [StructField]) {
        self.increment_indentation();
        self.write_identation();

        self.write("STRUCT_FIELDS");
        self.write_newline();

        walk_struct_fields(self, fields);

        self.decrement_indentation();
    }

    fn visit_trait_bounds(&mut self, bounds: &'_ [TypePathSegment]) {
        self.increment_indentation();
        self.write_identation();

        self.write("TRAIT_BOUNDS");
        self.write_newline();

        walk_trait_bounds(self, bounds);

        self.decrement_indentation();
    }

    fn visit_trait_item(&mut self, item: &'_ TraitItem) {
        self.increment_indentation();
        self.write_identation();

        self.write("TRAIT_ITEM");
        self.write_newline();

        walk_trait_item(self, item);

        self.decrement_indentation();
    }

    fn visit_trait_items(&mut self, items: &'_ [TraitItem]) {
        self.increment_indentation();
        self.write_identation();

        self.write("TRAIT_ITEMS");
        self.write_newline();

        walk_trait_items(self, items);

        self.decrement_indentation();
    }

    fn visit_tuple_field(&mut self, field: &'_ TupleField) {
        self.increment_indentation();
        self.write_identation();

        self.write("TUPLE_FIELD");
        self.write_newline();

        walk_tuple_field(self, field);

        self.decrement_indentation();
    }

    fn visit_tuple_fields(&mut self, fields: &'_ [TupleField]) {
        self.increment_indentation();
        self.write_identation();

        self.write("TUPLE_FIELDS");
        self.write_newline();

        walk_tuple_fields(self, fields);

        self.decrement_indentation();
    }

    fn visit_type(&mut self, ty: &'_ Type) {
        self.increment_indentation();
        self.write_identation();

        match ty {
            Type::Function { .. } => self.write("FUNCTION_TYPE"),
            Type::Tuple { .. } => self.write("TUPLE_TYPE"),
            Type::Path { .. } => self.write("PATH_TYPE"),
            Type::TraitObject { .. } => self.write("TRAIT_OBJECT_TYPE"),
            Type::Parenthesized { .. } => self.write("PARENTHESIZED_TYPE"),
            Type::WithQualifiedPath { .. } => self.write("WITH_QUALIFIED_PATH_TYPE"),
        }
        self.write(format!(" <{}>", ty.location()));
        self.write_newline();

        walk_type(self, ty);

        self.decrement_indentation();
    }

    fn visit_type_alias(&mut self, alias: &'_ TypeAlias) {
        self.increment_indentation();
        self.write_identation();

        self.write("TYPE_ALIAS");
        self.write_newline();

        walk_type_alias(self, alias);

        self.decrement_indentation();
    }

    fn visit_type_implementation(&mut self, implementation: &'_ Impl) {
        self.increment_indentation();
        self.write_identation();

        self.write("TYPE_IMPLEMENTATION");
        self.write_newline();

        walk_type_implementation(self, implementation);

        self.decrement_indentation();
    }

    fn visit_type_path(&mut self, path: &'_ TypePath) {
        self.increment_indentation();
        self.write_identation();

        self.write(format!("TYPE_PATH <{}>", path.location));
        self.write_newline();

        walk_type_path(self, path);

        self.decrement_indentation();
    }

    fn visit_type_path_segment(&mut self, segment: &'_ TypePathSegment) {
        self.increment_indentation();
        self.write_identation();

        self.write(format!("TYPE_PATH_SEGMENT <{}>", segment.location));
        self.write_newline();

        walk_type_path_segment(self, segment);

        self.decrement_indentation();
    }

    fn visit_visibility(&mut self, visibility: Visibility) {
        self.increment_indentation();
        self.write_identation();

        self.write("VISIBILITY: ");
        match visibility.location_of_pub() {
            Some(location) => self.write(format!("PUBLIC <{location}>")),
            None => self.write("PRIVATE"),
        }
        self.write_newline();

        self.decrement_indentation();
    }

    fn visit_where_predicates(&mut self, items: Option<&'_ [WherePredicate]>) {
        self.increment_indentation();
        self.write_identation();

        self.write("WHERE_CLAUSE");
        self.write_newline();

        walk_where_predicates(self, items);

        self.decrement_indentation();
    }

    fn visit_where_predicate(&mut self, item: &'_ WherePredicate) {
        self.increment_indentation();
        self.write_identation();

        self.write("WHERE_CLAUSE_ITEM");
        self.write_newline();

        walk_there_predicate(self, item);

        self.decrement_indentation();
    }
}

/// Serialize a module AST into a string.
#[must_use]
pub fn serialize_ast(module: &Module, identifier_interner: &IdentifierInterner) -> String {
    let mut serializer = Serializer::new(identifier_interner);
    serializer.visit_module(module);
    serializer.take_output()
}
