use std::path;

use crate::{
    token::RawToken, BinaryOperator, Docstring, Function, FunctionParameter, GenericArgument,
    GenericParameter, IdentifierAst, Item, Module, Path, PostfixOperator, PrefixOperator, TypeAst,
    TypePath, TypePathSegment, UntypedExpression, Visibility, WhereClauseItem,
};
use ry_interner::{Interner, Symbol};
use ry_source_file::{
    source_file::SourceFile,
    source_file_manager::{FileID, SourceFileManager},
    span::{Span, DUMMY_SPAN},
};

/// A struct that allows to serialize a Ry module into a string, for debug purposes.
#[derive(Debug)]
pub struct Serializer<'a> {
    /// An interner used to resolve symbols in an AST.
    interner: &'a Interner,

    /// A source file being serialized.
    source_file: &'a SourceFile<'a>,

    /// An ID of the source file being serialized.
    source_file_id: FileID,

    /// A source file manager.
    source_file_manager: &'a SourceFileManager<'a>,

    /// Current indentation level
    identation: usize,

    /// Symbols used for indentation.
    identation_symbols: &'a str,

    /// An output string produced,
    output: String,
}

impl<'a> Serializer<'a> {
    #[inline]
    #[must_use]
    pub fn new(
        interner: &'a Interner,
        source_file_id: FileID,
        source_file_manager: &'a SourceFileManager<'a>,
    ) -> Option<Self> {
        Some(Self {
            interner,
            source_file: source_file_manager.get_file_by_id(source_file_id)?,
            source_file_id,
            source_file_manager,
            identation: 0,
            identation_symbols: "\t",
            output: String::new(),
        })
    }

    /// Sets the symbols used for indentation.
    #[inline]
    #[must_use]
    pub fn with_identation_symbols(mut self, identation_symbols: &'a str) -> Self {
        self.identation_symbols = identation_symbols;

        self
    }

    /// Returns the path of the source file being serialized as a string slice.
    #[inline]
    #[must_use]
    pub fn filepath_str(&self) -> &'a str {
        self.source_file.path_str()
    }

    /// Returns the path of the source file being serialized.
    #[inline]
    #[must_use]
    pub const fn filepath(&self) -> &'a path::Path {
        self.source_file.path()
    }

    /// Returns the source content of the file being serialized.
    #[inline]
    #[must_use]
    pub const fn source(&self) -> &'a str {
        self.source_file.source()
    }

    /// Returns the length of the source content (in bytes).
    #[inline]
    #[must_use]
    pub const fn source_len(&self) -> usize {
        self.source_file.source().len()
    }

    /// Returns the ID of the source file being serialized.
    #[inline]
    #[must_use]
    pub const fn file_id(&self) -> FileID {
        self.source_file_id
    }

    /// Returns the interner used to resolve symbols in the AST of the module being serialized.
    #[inline]
    #[must_use]
    pub const fn interner(&self) -> &'a Interner {
        self.interner
    }

    /// Returns the source file manager.
    #[inline]
    #[must_use]
    pub fn file_manager(&self) -> &'a SourceFileManager<'a> {
        self.source_file_manager
    }

    /// Returns the current indentation level.
    #[inline]
    #[must_use]
    pub const fn identation(&self) -> usize {
        self.identation
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

    /// Returns the symbols used for indentation.
    #[inline]
    #[must_use]
    pub const fn identation_symbols(&self) -> &'a str {
        self.identation_symbols
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
        for _ in 0..self.identation() {
            self.write(self.identation_symbols());
        }
    }

    pub fn write_node_name<S>(&mut self, node_name: S)
    where
        S: AsRef<str>,
    {
        self.write(node_name);
    }

    pub fn write_node_name_with_span<S>(&mut self, node_name: S, span: Span)
    where
        S: AsRef<str>,
    {
        self.write(node_name);
        self.write("@");

        span.serialize(self);
    }

    pub fn serialize_key_value_pair<S, Se>(&mut self, key: S, value: &Se)
    where
        S: AsRef<str>,
        Se: Serialize<'a>,
    {
        self.write_newline();
        self.write_identation();
        self.write(key);
        self.write(": ");
        value.serialize(self);
    }

    pub fn serialize_item<S>(&mut self, item: &S)
    where
        S: Serialize<'a>,
    {
        self.write_newline();
        self.write_identation();
        item.serialize(self);
    }

    pub fn serialize_items<S>(&mut self, items: &Vec<S>)
    where
        S: Serialize<'a>,
    {
        self.increment_indentation();

        for item in items {
            self.serialize_item(item);
        }

        self.decrement_indentation();
    }

    pub fn serialize_key_list_value_pair<S, Se>(&mut self, key: S, items: &Vec<Se>)
    where
        S: AsRef<str>,
        Se: Serialize<'a>,
    {
        self.write_newline();
        self.write_identation();
        self.write(key);
        self.write(": ");
        self.serialize_items(items);
    }

    /// Returns the output string produced.
    #[inline]
    #[must_use]
    pub fn output(&self) -> &str {
        &self.output
    }
}

pub trait Serialize<'a> {
    fn serialize(&self, serializer: &mut Serializer<'a>);
}

impl<'a> Serialize<'a> for Span {
    fn serialize(&self, serializer: &mut Serializer<'a>) {
        match self {
            &DUMMY_SPAN => serializer.write("DUMMY"),
            _ => serializer.write(
                if self.start() >= self.end() || self.file_id() != serializer.file_id() {
                    "INVALID".to_owned()
                } else {
                    format!("{}..{}", self.start(), self.end())
                },
            ),
        }
    }
}

impl<'a> Serialize<'a> for Symbol {
    fn serialize(&self, serializer: &mut Serializer<'a>) {
        serializer.write(
            serializer
                .interner()
                .resolve(*self)
                .unwrap_or_else(|| panic!("Symbol {self} cannot be resolved")),
        );
    }
}

impl<'a> Serialize<'a> for IdentifierAst {
    fn serialize(&self, serializer: &mut Serializer<'a>) {
        serializer.write("IDENTIFIER(`");
        self.symbol.serialize(serializer);
        serializer.write("`)@");
        self.span.serialize(serializer);
    }
}

impl<'a> Serialize<'a> for Path {
    fn serialize(&self, serializer: &mut Serializer<'a>) {
        serializer.write_node_name_with_span("PATH", self.span);
        serializer.serialize_items(&self.identifiers);
    }
}

impl<'a> Serialize<'a> for TypePath {
    fn serialize(&self, serializer: &mut Serializer<'a>) {
        serializer.write_node_name_with_span("TYPE_PATH", self.span);
        serializer.serialize_items(&self.segments);
    }
}

impl<'a> Serialize<'a> for TypePathSegment {
    fn serialize(&self, serializer: &mut Serializer<'a>) {
        serializer.write_node_name_with_span("TYPE_PATH_SEGMENT", self.span);
        serializer.serialize_key_value_pair("PATH", &self.path);
        if let Some(generic_arguments) = &self.generic_arguments {
            serializer.serialize_key_list_value_pair("GENERIC_ARGUMENTS", generic_arguments);
        }
    }
}

impl<'a> Serialize<'a> for GenericArgument {
    fn serialize(&self, serializer: &mut Serializer<'a>) {
        match self {
            Self::Type(ty) => {
                serializer.write_node_name_with_span("GENERIC_ARGUMENT", ty.span());
                serializer.serialize_key_value_pair("TYPE", ty);
            }
            Self::AssociatedType { name, value } => {
                serializer.write_node_name_with_span(
                    "NAMED_GENERIC_ARGUMENT",
                    Span::new(name.span.start(), value.span().end(), serializer.file_id()),
                );
                serializer.serialize_key_value_pair("NAME", name);
                serializer.serialize_key_value_pair("VALUE", value);
            }
        }
    }
}

impl<'a> Serialize<'a> for BinaryOperator {
    fn serialize(&self, serializer: &mut Serializer<'a>) {
        serializer.write_node_name_with_span(format!("{}", RawToken::from(self.raw)), self.span);
    }
}

impl<'a> Serialize<'a> for PostfixOperator {
    fn serialize(&self, serializer: &mut Serializer<'a>) {
        serializer.write_node_name_with_span(format!("{}", RawToken::from(self.raw)), self.span);
    }
}

impl<'a> Serialize<'a> for PrefixOperator {
    fn serialize(&self, serializer: &mut Serializer<'a>) {
        serializer.write_node_name_with_span(format!("{}", RawToken::from(self.raw)), self.span);
    }
}

impl<'a> Serialize<'a> for UntypedExpression {
    fn serialize(&self, serializer: &mut Serializer<'a>) {
        match self {
            Self::As { span, left, right } => {
                serializer.write_node_name_with_span("CAST_EXPRESSION", *span);
                serializer.serialize_key_value_pair("LEFT", &**left);
                serializer.serialize_key_value_pair("RIGHT", right);
            }
            Self::Binary {
                span,
                left,
                operator,
                right,
            } => {
                serializer.write_node_name_with_span("BINARY_EXPRESSION", *span);
                serializer.serialize_key_value_pair("LEFT", &**left);
                serializer.serialize_key_value_pair("OPERATOR", operator);
                serializer.serialize_key_value_pair("RIGHT", &**right);
            }
            Self::Call {
                span,
                left,
                arguments,
            } => {
                serializer.write_node_name_with_span("CALL_EXPRESSION", *span);
                serializer.serialize_key_value_pair("LEFT", &**left);
                serializer.serialize_key_list_value_pair("ARGUMENTS", arguments);
            }
            Self::Function {
                span,
                parameters,
                return_type,
                block,
            } => {
                serializer.write_node_name_with_span("FUNCTION_EXPRESSION", *span);
                serializer.serialize_key_list_value_pair("PARAMETERS", parameters);

                if let Some(return_type) = return_type {
                    serializer.serialize_key_value_pair("RETURN_TYPE", return_type);
                }

                serializer.serialize_key_value_pair("STATEMENTS_BLOCK", block);
            }
            Self::GenericArguments {
                span,
                left,
                generic_arguments,
            } => {
                serializer.write_node_name_with_span("GENERIC_ARGUMENTS", *span);
                serializer.serialize_key_value_pair("LEFT", &**left);
                serializer.serialize_key_list_value_pair("GENERIC_ARGUMENTS", generic_arguments);
            }
            Self::Identifier(symbol) => symbol.serialize(serializer),
            Self::If {
                span,
                if_blocks,
                r#else,
            } => {
                serializer.write_node_name_with_span("IF_EXPRESSION", *span);
                serializer.serialize_key_list_value_pair("IF_BLOCKS", if_blocks);
                if let Some(r#else) = r#else {
                    serializer.serialize_key_list_value_pair("ELSE_BLOCK", r#else);
                }
            }
            Self::List { span, elements } => {
                serializer.write_node_name_with_span("LIST_EXPRESSION", *span);
                serializer.serialize_key_list_value_pair("ELEMENTS", elements);
            }
            Self::Parenthesized { span, inner } => {
                serializer.write_node_name_with_span("PARENTHESIZED_EXPRESSION", *span);
                serializer.serialize_key_value_pair("INNER", &**inner);
            }
            Self::Postfix {
                span,
                inner,
                operator,
            } => {
                serializer.write_node_name_with_span("POSTFIX_EXPRESSION", *span);
                serializer.serialize_key_value_pair("LEFT", &**inner);
                serializer.serialize_key_value_pair("OPERATOR", operator);
            }
            Self::Prefix {
                span,
                inner,
                operator,
            } => {
                serializer.write_node_name_with_span("PREFIX_EXPRESSION", *span);
                serializer.serialize_key_value_pair("OPERATOR", operator);
                serializer.serialize_key_value_pair("INNER", &**inner);
            }
            Self::FieldAccess { span, left, right } => {
                serializer.write_node_name_with_span("FIELD_ACCESS_EXPRESSION", *span);
                serializer.serialize_key_value_pair("LEFT", &**left);
                serializer.serialize_key_value_pair("RIGHT", right);
            }
            Self::StatementsBlock { span, block } => {
                serializer.write_node_name_with_span("BLOCK_EXPRESSION", *span);
                serializer.serialize_key_list_value_pair("STATEMENTS_BLOCK", block);
            }
            Self::Struct { span, left, fields } => {
                serializer.write_node_name_with_span("STRUCT_EXPRESSION", *span);
                serializer.serialize_key_value_pair("LEFT", &**left);
                serializer.serialize_key_list_value_pair("FIELDS", fields);
            }
            Self::Tuple { span, elements } => {
                serializer.write_node_name_with_span("TUPLE_EXPRESSION", *span);
                serializer.serialize_key_list_value_pair("ELEMENTS", elements);
            }
            Self::While {
                span,
                condition,
                body,
            } => {
                serializer.write_node_name_with_span("WHILE_EXPRESSION", *span);
                serializer.serialize_key_value_pair("CONDITION", &**condition);
                serializer.serialize_key_list_value_pair("BODY_STATEMENTS_BLOCK", body);
            }
            Self::Literal(literal) => literal.serialize(serializer),
            Self::Match {
                span,
                expression,
                block,
            } => {
                serializer.write_node_name_with_span("MATCH_EXPRESSION", *span);
                serializer.serialize_key_value_pair("EXPRESSION", &**expression);
                serializer.serialize_key_list_value_pair("BLOCK", block);
            }
        }
    }
}

impl<'a> Serialize<'a> for TypeAst {
    fn serialize(&self, serializer: &mut Serializer<'a>) {
        match self {
            Self::Function {
                span,
                parameter_types,
                return_type,
            } => {
                serializer.write_node_name_with_span("FUNCTION_TYPE", *span);
                serializer.serialize_key_list_value_pair("PARAMETER_TYPES", parameter_types);
                serializer.serialize_key_value_pair("RETURN_TYPE", &**return_type);
            }
            Self::Parenthesized { span, inner } => {
                serializer.write_node_name_with_span("PARENTHESIZED_TYPE", *span);
                serializer.serialize_key_value_pair("TYPE", &**inner);
            }
            Self::Path(path) => {
                path.serialize(serializer);
            }
            Self::TraitObject { span, bounds } => {
                serializer.write_node_name_with_span("TRAIT_OBJECT_TYPE", *span);
                serializer.serialize_key_list_value_pair("BOUNDS", bounds);
            }
            Self::Tuple {
                span,
                element_types,
            } => {
                serializer.write_node_name_with_span("TUPLE_TYPE", *span);
                serializer.serialize_key_list_value_pair("ELEMENT_TYPES", element_types);
            }
            Self::WithQualifiedPath {
                span,
                left,
                right,
                segments,
            } => {
                serializer.write_node_name_with_span("TYPE_WITH_QUALIFIED_PATH", *span);
                serializer.serialize_key_value_pair("LEFT", &**left);
                serializer.serialize_key_value_pair("RIGHT", right);
                serializer.serialize_key_list_value_pair("SEGMENTS", segments);
            }
        }
    }
}

impl<'a> Serialize<'a> for Visibility {
    fn serialize(&self, serializer: &mut Serializer<'a>) {
        match self.span_of_pub() {
            Some(pub_span) => {
                serializer.write("PUBLIC@");
                pub_span.serialize(serializer);
            }
            None => {
                serializer.write("PRIVATE");
            }
        }
    }
}

impl<'a> Serialize<'a> for GenericParameter {
    fn serialize(&self, serializer: &mut Serializer<'a>) {
        serializer.write_node_name("GENERIC_PARAMETER");
        serializer.serialize_key_value_pair("NAME", &self.name);

        if let Some(bounds) = &self.bounds {
            serializer.serialize_key_list_value_pair("BOUNDS", bounds);
        }

        if let Some(default) = &self.default_value {
            serializer.serialize_key_value_pair("DEFAULT", default);
        }
    }
}

impl<'a> Serialize<'a> for WhereClauseItem {
    fn serialize(&self, serializer: &mut Serializer<'a>) {
        match self {
            Self::Eq { left, right } => {
                serializer.write_node_name("WHERE_CLAUSE_ITEM_EQ");
                serializer.serialize_key_value_pair("LEFT", left);
                serializer.serialize_key_value_pair("RIGHT", right);
            }
            Self::Satisfies { ty, bounds } => {
                serializer.write_node_name("WHERE_CLAUSE_ITEM_SATISFIES");
                serializer.serialize_key_value_pair("TYPE", ty);
                serializer.serialize_key_list_value_pair("BOUNDS", bounds);
            }
        }
    }
}

impl<'a> Serialize<'a> for FunctionParameter {
    fn serialize(&self, serializer: &mut Serializer<'a>) {
        match self {
            Self::Just(parameter) => {
                serializer.write_node_name("FUNCTION_PARAMETER");

                serializer.serialize_key_value_pair("NAME", &parameter.name);
                serializer.serialize_key_value_pair("TYPE", &parameter.ty);

                if let Some(default_value) = parameter.default_value {
                    serializer.serialize_key_value_pair("DEFAULT_VALUE", &default_value);
                }
            }
            Self::Self_(parameter) => {
                serializer.write_node_name("SELF_PARAMETER");

                serializer.serialize_key_value_pair("SELF_SPAN", &parameter.self_span);

                if let Some(ty) = parameter.ty {
                    serializer.serialize_key_value_pair("TYPE", &ty);
                }
            }
        }
    }
}

impl<'a> Serialize<'a> for Function {
    fn serialize(&self, serializer: &mut Serializer<'a>) {
        serializer.write_node_name("FUNCTION");
        serializer.serialize_key_value_pair("VISIBILITY", &self.visibility);
        serializer.serialize_key_value_pair("NAME", &self.name);

        if let Some(generic_parameters) = &self.generic_parameters {
            serializer.serialize_key_list_value_pair("GENERIC_PARAMETERS", generic_parameters);
        }

        serializer.serialize_key_list_value_pair("PARAMETERS", &self.parameters);

        if let Some(return_type) = &self.return_type {
            serializer.serialize_key_value_pair("RETURN_TYPE", return_type);
        }

        if let Some(where_clause) = &self.where_clause {
            serializer.serialize_key_list_value_pair("WHERE_CLAUSE", where_clause);
        }
    }
}

impl<'a> Serialize<'a> for Item {
    fn serialize(&self, serializer: &mut Serializer<'a>) {
        match self {
            Self::Enum {
                visibility,
                name,
                generic_parameters,
                where_clause,
                items,
            } => {
                serializer.write_node_name("ENUM");
                serializer.serialize_key_value_pair("VISIBILITY", visibility);
                serializer.serialize_key_value_pair("NAME", name);
                if let Some(generic_parameters) = generic_parameters {
                    serializer
                        .serialize_key_list_value_pair("GENERIC_PARAMETERS", generic_parameters);
                }
                if let Some(where_clause) = where_clause {
                    serializer.serialize_key_list_value_pair("WHERE_CLAUSE", where_clause);
                }
                serializer.serialize_key_list_value_pair("ITEMS", items);
            }
            Self::Function(function) => function.serialize(serializer),
            Self::Impl {
                visibility,
                generic_parameters,
                r#type,
                r#trait,
                where_clause,
                items,
            } => {
                serializer.write_node_name("IMPL");
                serializer.serialize_key_value_pair("VISIBILITY", visibility);

                if let Some(generic_parameters) = generic_parameters {
                    serializer
                        .serialize_key_list_value_pair("GENERIC_PARAMETERS", generic_parameters);
                }
                serializer.serialize_key_value_pair("TYPE", r#type);

                if let Some(r#trait) = r#trait {
                    serializer.serialize_key_value_pair("TRAIT", r#trait);
                }

                if let Some(where_clause) = where_clause {
                    serializer.serialize_key_list_value_pair("WHERE_CLAUSE", where_clause);
                }
            }
            Self::Struct {
                visibility,
                name,
                generic_parameters,
                where_clause,
                fields,
            } => {
                serializer.write_node_name("STRUCT");
                serializer.serialize_key_value_pair("VISIBILITY", visibility);
                serializer.serialize_key_value_pair("NAME", name);
                if let Some(generic_parameters) = generic_parameters {
                    serializer
                        .serialize_key_list_value_pair("GENERIC_PARAMETERS", generic_parameters);
                }
                if let Some(where_clause) = where_clause {
                    serializer.serialize_key_list_value_pair("WHERE_CLAUSE", where_clause);
                }

                serializer.serialize_key_list_value_pair(
                    "FIELDS",
                    &fields.into_iter().map(|f| f.value).collect::<Vec<_>>(),
                );
            }
            Self::Trait {
                visibility,
                name,
                generic_parameters,
                where_clause,
                items,
            } => {
                serializer.write_node_name("TRAIT");
                serializer.serialize_key_value_pair("VISIBILITY", visibility);
                serializer.serialize_key_value_pair("NAME", name);
                if let Some(generic_parameters) = generic_parameters {
                    serializer
                        .serialize_key_list_value_pair("GENERIC_PARAMETERS", generic_parameters);
                }
                if let Some(where_clause) = where_clause {
                    serializer.serialize_key_list_value_pair("WHERE_CLAUSE", where_clause);
                }

                serializer.serialize_key_list_value_pair(
                    "ITEMS",
                    &items.into_iter().map(|i| i.value).collect::<Vec<_>>(),
                );
            }
            Self::TupleLikeStruct {
                visibility,
                name,
                generic_parameters,
                where_clause,
                fields,
            } => {
                serializer.write_node_name("TUPLE_LIKE_STRUCT");

                serializer.serialize_key_value_pair("VISIBILITY", visibility);
                serializer.serialize_key_value_pair("NAME", name);

                if let Some(generic_parameters) = generic_parameters {
                    serializer
                        .serialize_key_list_value_pair("GENERIC_PARAMETERS", generic_parameters);
                }

                if let Some(where_clause) = where_clause {
                    serializer.serialize_key_list_value_pair("WHERE_CLAUSE", where_clause);
                }

                serializer.serialize_key_list_value_pair("FIELDS", fields);
            }
            Self::TypeAlias(alias) => alias.serialize(serializer),
            Self::Use { visibility, path } => {
                serializer.write_node_name("USE");
                serializer.serialize_key_value_pair("VISIBILITY", visibility);
                serializer.serialize_key_value_pair("PATH", path);
            }
        }
    }
}

impl<'a> Serialize<'a> for &path::Path {
    fn serialize(&self, serializer: &mut Serializer<'a>) {
        serializer.write(self.to_str().expect("Invalid UTF-8 in filepath"));
    }
}

impl<'a> Serialize<'a> for Module<'a> {
    fn serialize(&self, serializer: &mut Serializer<'a>) {
        serializer.write_node_name("MODULE");
        serializer.serialize_key_value_pair("FILEPATH", &self.filepath);
        serializer.serialize_key_list_value_pair(
            "ITEMS",
            &self.items.into_iter().map(|i| i.value).collect::<Vec<_>>(),
        );
    }
}
