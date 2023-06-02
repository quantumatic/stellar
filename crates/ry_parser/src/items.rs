use crate::{
    expression::ExpressionParser,
    macros::parse_list,
    path::PathParser,
    r#type::{GenericParametersParser, TypeParser, WhereClauseParser},
    statement::StatementsBlockParser,
    Cursor, OptionalParser, Parse,
};
use ry_ast::{
    span::Span, token::RawToken, Docstring, Documented, EnumItem, Function, FunctionParameter,
    Identifier, Item, ItemKind, Items, StructField, Token, TraitItem, TupleField, TypeAlias,
    Visibility, WithDocComment,
};
use ry_diagnostics::{expected, parser::ParseDiagnostic, Report};

struct ImportParser {
    pub(crate) visibility: Visibility,
}

struct StructItemParser {
    pub(crate) visibility: Visibility,
}

struct StructFieldsParser;

struct StructFieldParser;

struct FunctionParser {
    pub(crate) visibility: Visibility,
}

pub(crate) struct FunctionParameterParser;

struct TypeAliasParser {
    pub(crate) visibility: Visibility,
}

struct TraitItemParser {
    pub(crate) visibility: Visibility,
}

struct TraitItemsParser {
    pub(crate) name_span: Span,
    pub(crate) item_kind: ItemKind,
}

struct ImplItemParser {
    pub(crate) visibility: Visibility,
}

struct EnumParser {
    pub(crate) visibility: Visibility,
}

struct EnumItemParser;

struct EnumItemTupleParser {
    pub(crate) name: Identifier,
}

struct EnumItemStructParser {
    pub(crate) name: Identifier,
}

pub(crate) struct ItemsParser {
    pub(crate) first_docstring: Docstring,
}

pub(crate) struct ItemParser;

impl Parse for ImportParser {
    type Output = Option<Item>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token();

        let path = PathParser.parse_with(cursor)?;
        cursor.consume(Token![;], "import")?;

        Some(Item::Import {
            visibility: self.visibility,
            path,
        })
    }
}

impl Parse for StructFieldParser {
    type Output = Option<StructField>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let mut visibility = Visibility::private();

        if *cursor.next.unwrap() == Token![pub] {
            cursor.next_token();
            visibility = Visibility::public(cursor.current.span());
        }

        let name = cursor.consume_identifier("struct field")?;

        cursor.consume(Token![:], "struct field")?;

        let r#type = TypeParser.parse_with(cursor)?;

        Some(StructField {
            visibility,
            name,
            r#type,
        })
    }
}

impl Parse for StructFieldsParser {
    type Output = Option<Vec<Documented<StructField>>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.consume(Token!['{'], "struct fields")?;

        let fields = parse_list!(
            cursor,
            "struct fields",
            Token!['}'],
            || -> Option<Documented<StructField>> {
                let docstring = cursor.consume_docstring();
                Some(
                    StructFieldParser
                        .parse_with(cursor)?
                        .with_doc_comment(docstring),
                )
            }
        );

        cursor.next_token(); // `}`

        Some(fields)
    }
}

impl Parse for StructItemParser {
    type Output = Option<Item>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token();

        let name = cursor.consume_identifier("struct name")?;

        let generic_parameters = GenericParametersParser.optionally_parse_with(cursor)?;

        let where_clause = WhereClauseParser.optionally_parse_with(cursor)?;

        let fields = StructFieldsParser.parse_with(cursor)?;

        Some(Item::Struct {
            visibility: self.visibility,
            name,
            generic_parameters,
            where_clause,
            fields,
        })
    }
}

impl Parse for FunctionParameterParser {
    type Output = Option<FunctionParameter>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let name = cursor.consume_identifier("function parameter name")?;

        cursor.consume(Token![:], "function parameter name")?;

        let r#type = TypeParser.parse_with(cursor)?;

        let mut default_value = None;

        if *cursor.next.unwrap() == Token![=] {
            cursor.next_token();
            default_value = Some(ExpressionParser::default().parse_with(cursor)?);
        }

        Some(FunctionParameter {
            name,
            r#type,
            default_value,
        })
    }
}

impl Parse for FunctionParser {
    type Output = Option<Function>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token();

        let name = cursor.consume_identifier("function name")?;

        let generic_parameters = GenericParametersParser.optionally_parse_with(cursor)?;

        cursor.consume(Token!['('], "function")?;

        let parameters = parse_list!(cursor, "function parameters", Token![')'], || {
            FunctionParameterParser.parse_with(cursor)
        });

        cursor.next_token();

        let mut return_type = None;

        if *cursor.next.unwrap() == Token![:] {
            cursor.next_token();
            return_type = Some(TypeParser.parse_with(cursor)?);
        }

        let where_clause = WhereClauseParser.optionally_parse_with(cursor)?;

        Some(Function {
            visibility: self.visibility,
            name,
            generic_parameters,
            parameters,
            return_type,
            where_clause,
            body: match cursor.next.unwrap() {
                Token![;] => {
                    cursor.next_token();

                    None
                }
                Token!['{'] => Some(StatementsBlockParser.parse_with(cursor)?),
                _ => {
                    cursor.next_token();

                    cursor.diagnostics.push(
                        ParseDiagnostic::UnexpectedTokenError {
                            got: cursor.current.clone(),
                            expected: expected!(Token![;], Token!['(']),
                            node: "end of function".to_owned(),
                        }
                        .build(),
                    );

                    None
                }
            },
        })
    }
}

impl Parse for TraitItemsParser {
    type Output = Option<(Vec<Documented<TraitItem>>, bool)>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let mut items = vec![];

        while *cursor.next.unwrap() != Token!['}'] {
            let doc = cursor.consume_docstring();

            let visibility = if *cursor.next.unwrap() == Token![pub] {
                cursor.next_token();
                Visibility::public(cursor.current.span())
            } else {
                Visibility::private()
            };

            items.push(match cursor.next.unwrap() {
                Token![fun] => Some(
                    TraitItem::AssociatedFunction(
                        FunctionParser { visibility }.parse_with(cursor)?,
                    )
                    .with_doc_comment(doc),
                ),
                Token![type] => Some(
                    TraitItem::TypeAlias(TypeAliasParser { visibility }.parse_with(cursor)?)
                        .with_doc_comment(doc),
                ),
                RawToken::EndOfFile => {
                    cursor.diagnostics.push(
                        ParseDiagnostic::EOFInsteadOfCloseBraceForItemError {
                            item_kind: self.item_kind,
                            item_name_span: self.name_span,
                            at: cursor.current.span(),
                        }
                        .build(),
                    );
                    return Some((items, true));
                }
                _ => {
                    cursor.diagnostics.push(
                        ParseDiagnostic::UnexpectedTokenError {
                            got: cursor.next.clone(),
                            expected: expected!(Token![fun], Token![type]),
                            node: "trait item".to_owned(),
                        }
                        .build(),
                    );
                    None
                }
            }?);
        }

        Some((items, false))
    }
}

impl Parse for TypeAliasParser {
    type Output = Option<TypeAlias>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token();

        let name = cursor.consume_identifier("type alias")?;
        let generic_parameters = GenericParametersParser.optionally_parse_with(cursor)?;

        let value = if *cursor.next.unwrap() == Token![=] {
            cursor.next_token();

            Some(TypeParser.parse_with(cursor)?)
        } else {
            None
        };

        cursor.consume(Token![;], "type alias")?;

        Some(TypeAlias {
            visibility: self.visibility,
            name,
            generic_parameters,
            value,
        })
    }
}

impl Parse for TraitItemParser {
    type Output = Option<Item>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token();

        let name = cursor.consume_identifier("trait name in trait declaration")?;

        let generic_parameters = GenericParametersParser.optionally_parse_with(cursor)?;

        let where_clause = WhereClauseParser.optionally_parse_with(cursor)?;

        cursor.consume(Token!['{'], "trait declaration")?;

        let items = TraitItemsParser {
            name_span: name.span(),
            item_kind: ItemKind::Trait,
        }
        .parse_with(cursor)?;

        if !items.1 {
            cursor.consume(Token!['}'], "trait declaration")?;
        }

        Some(Item::Trait {
            visibility: self.visibility,
            name,
            generic_parameters,
            where_clause,
            items: items.0,
        })
    }
}

impl Parse for ImplItemParser {
    type Output = Option<Item>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token();
        let impl_span = cursor.current.span();

        let generic_parameters = GenericParametersParser.optionally_parse_with(cursor)?;

        let mut r#type = TypeParser.parse_with(cursor)?;
        let mut r#trait = None;

        if *cursor.next.unwrap() == Token![for] {
            cursor.next_token();

            r#trait = Some(r#type);
            r#type = TypeParser.parse_with(cursor)?;
        }

        let where_clause = WhereClauseParser.optionally_parse_with(cursor)?;

        cursor.consume(Token!['{'], "type implementation")?;

        let items = TraitItemsParser {
            name_span: impl_span,
            item_kind: ItemKind::Impl,
        }
        .parse_with(cursor)?;

        if !items.1 {
            cursor.consume(Token!['}'], "type implementation")?;
        }

        Some(Item::Impl {
            visibility: self.visibility,
            generic_parameters,
            r#type,
            r#trait,
            where_clause,
            items: items.0,
        })
    }
}

impl Parse for EnumParser {
    type Output = Option<Item>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token();

        let name = cursor.consume_identifier("enum name")?;

        let generic_parameters = GenericParametersParser.optionally_parse_with(cursor)?;

        cursor.consume(Token!['{'], "enum")?;

        let items = parse_list!(
            cursor,
            "enum items",
            Token!['}'],
            || -> Option<Documented<EnumItem>> {
                let doc = cursor.consume_docstring();
                Some(EnumItemParser.parse_with(cursor)?.with_doc_comment(doc))
            }
        );

        cursor.next_token(); // `}`

        let where_clause = WhereClauseParser.optionally_parse_with(cursor)?;

        Some(Item::Enum {
            visibility: self.visibility,
            name,
            generic_parameters,
            where_clause,
            items,
        })
    }
}

impl Parse for EnumItemParser {
    type Output = Option<EnumItem>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let name = cursor.consume_identifier("enum item")?;

        match cursor.next.unwrap() {
            Token!['{'] => EnumItemStructParser { name }.parse_with(cursor),
            Token!['('] => EnumItemTupleParser { name }.parse_with(cursor),
            _ => Some(EnumItem::Identifier(name)),
        }
    }
}

impl Parse for EnumItemStructParser {
    type Output = Option<EnumItem>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let fields = StructFieldsParser.parse_with(cursor)?;

        Some(EnumItem::Struct {
            name: self.name,
            fields,
        })
    }
}

impl Parse for EnumItemTupleParser {
    type Output = Option<EnumItem>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token(); // `(`

        let fields = parse_list!(
            cursor,
            "enum item tuple",
            Token![')'],
            || -> Option<TupleField> {
                let visibility = if cursor.next.unwrap() == &Token![pub] {
                    cursor.next_token();
                    Visibility::public(cursor.current.span())
                } else {
                    Visibility::private()
                };

                let r#type = TypeParser.parse_with(cursor)?;

                Some(TupleField { visibility, r#type })
            }
        );

        cursor.next_token(); // `)`

        Some(EnumItem::Tuple {
            name: self.name,
            fields,
        })
    }
}

impl Parse for ItemsParser {
    type Output = Items;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let mut items = vec![];
        let mut docstring = self.first_docstring;

        while *cursor.next.unwrap() != RawToken::EndOfFile {
            if let Some(item) = ItemParser.parse_with(cursor) {
                items.push(item.with_doc_comment(docstring));
            }

            docstring = cursor.consume_docstring();
        }

        items
    }
}

impl ItemParser {
    fn go_to_next_item(cursor: &mut Cursor<'_>) {
        loop {
            match cursor.next.unwrap() {
                Token![enum]
                | Token![import]
                | Token![struct]
                | Token![trait]
                | Token![fun]
                | Token![type]
                | Token![impl]
                | RawToken::EndOfFile => break,
                _ => cursor.next_token(),
            }
        }
    }
}

macro_rules! go_to_next_valid_item {
    ($cursor:ident, $item:expr) => {
        if let Some(item) = $item {
            item
        } else {
            Self::go_to_next_item($cursor);
            return None;
        }
    };
}

impl Parse for ItemParser {
    type Output = Option<Item>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let mut visibility = Visibility::private();

        if *cursor.next.unwrap() == Token![pub] {
            visibility = Visibility::public(cursor.next.span());
            cursor.next_token();
        }

        Some(match cursor.next.unwrap() {
            Token![enum] => {
                go_to_next_valid_item!(cursor, EnumParser { visibility }.parse_with(cursor))
            }
            Token![import] => {
                go_to_next_valid_item!(cursor, ImportParser { visibility }.parse_with(cursor))
            }
            Token![struct] => {
                go_to_next_valid_item!(cursor, StructItemParser { visibility }.parse_with(cursor))
            }
            Token![trait] => {
                go_to_next_valid_item!(cursor, TraitItemParser { visibility }.parse_with(cursor))
            }
            Token![fun] => Item::Function(go_to_next_valid_item!(
                cursor,
                FunctionParser { visibility }.parse_with(cursor)
            )),
            Token![impl] => {
                go_to_next_valid_item!(cursor, ImplItemParser { visibility }.parse_with(cursor))
            }
            Token![type] => Item::TypeAlias(go_to_next_valid_item!(
                cursor,
                TypeAliasParser { visibility }.parse_with(cursor)
            )),
            _ => {
                cursor.diagnostics.push(
                    ParseDiagnostic::UnexpectedTokenError {
                        got: cursor.next.clone(),
                        expected: expected!(
                            Token![import],
                            Token![fun],
                            Token![trait],
                            Token![enum],
                            Token![struct],
                            Token![impl],
                            Token![type],
                            RawToken::EndOfFile
                        ),
                        node: "item".to_owned(),
                    }
                    .build(),
                );

                loop {
                    match cursor.next.unwrap() {
                        Token![enum]
                        | Token![import]
                        | Token![struct]
                        | Token![trait]
                        | Token![fun]
                        | Token![type]
                        | Token![impl]
                        | RawToken::EndOfFile => break,
                        _ => cursor.next_token(),
                    }
                }
                return None;
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ItemParser;
    use crate::macros::parse_test;

    parse_test!(ItemParser, function1, "fun test();");
    parse_test!(ItemParser, function2, "fun test[A](a: A): A { a }");
    parse_test!(
        ItemParser,
        function3,
        "fun unwrap[T, B: Option[T]](a: B): T { a.unwrap() }"
    );
    parse_test!(ItemParser, impl1, "impl[T] NotOption for T {}");
    parse_test!(
        ItemParser,
        impl2,
        "impl[T] Into[Option[M]] for Tuple[T, M] where M: Into[T] {}"
    );
    parse_test!(ItemParser, single_import, "import test;");
    parse_test!(ItemParser, imports, "import test; import test2.test;");
    parse_test!(ItemParser, empty_struct, "struct test {}");
    parse_test!(
        ItemParser,
        r#struct1,
        "struct Point[T: Numeric] { pub x: T, pub y: T }"
    );
    parse_test!(
        ItemParser,
        r#struct2,
        "struct Lexer[S] where S: Iterator[char] { contents: S }"
    );
    parse_test!(ItemParser, empty_trait, "trait test {}");
    parse_test!(ItemParser, trait1, "trait test { fun f(); }");
    parse_test!(
        ItemParser,
        trait2,
        "trait Into[T] { fun into(self: Self): T; }"
    );
    parse_test!(ItemParser, empty_type_alias, "type A;");
    parse_test!(ItemParser, type_alias1, "type B = Option[i32];");
    parse_test!(ItemParser, type_alias2, "type B[T] = Option[T];");
    parse_test!(ItemParser, no_variants, "enum test {}");
    parse_test!(ItemParser, single_variant, "enum test { a, b, c }");
    parse_test!(ItemParser, enum1, "enum Result[T, E] { Some(T), Err(E) }");
    parse_test!(ItemParser, enum2, "enum Option[T] { Some(T), None }");
    parse_test!(
        ItemParser,
        enum3,
        "pub enum UserPrincipal {
        Full {
            email: string,
            phone_number: PhoneNumber,
        },
        EmailOnly { email: string },
        PhoneNumberOnly { phone_number: PhoneNumber },
    }"
    );
}
