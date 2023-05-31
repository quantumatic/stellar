use crate::{
    error::{expected, ParseError, ParseResult},
    expression::ExpressionParser,
    macros::parse_list,
    path::PathParser,
    r#type::{GenericParametersParser, TypeParser, WhereClauseParser},
    statement::StatementsBlockParser,
    Cursor, OptionalParser, Parse,
};
use ry_ast::{
    token::RawToken, Docstring, Documented, EnumItem, Function, FunctionParameter, Identifier,
    Item, Items, StructField, Token, TraitItem, TupleField, TypeAlias, Visibility, WithDocComment,
};

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

struct FunctionParameterParser;

struct TypeAliasParser {
    pub(crate) visibility: Visibility,
}

struct TraitItemParser {
    pub(crate) visibility: Visibility,
}

struct TraitItemsParser;

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
    type Output = Item;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.next_token();

        let path = PathParser.parse_with(cursor)?;
        cursor.consume(Token![;], "import")?;

        Ok(Item::Import {
            visibility: self.visibility,
            path,
        })
    }
}

impl Parse for StructFieldParser {
    type Output = StructField;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        let mut visibility = Visibility::private();

        if *cursor.next.unwrap() == Token![pub] {
            cursor.next_token();
            visibility = Visibility::public(cursor.current.span());
        }

        let name = cursor.consume_identifier("struct field")?;

        cursor.consume(Token![:], "struct field")?;

        let r#type = TypeParser.parse_with(cursor)?;

        Ok(StructField {
            visibility,
            name,
            r#type,
        })
    }
}

impl Parse for StructFieldsParser {
    type Output = Vec<Documented<StructField>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.consume(Token!['{'], "struct fields")?;

        let fields = parse_list!(
            cursor,
            "struct fields",
            Token!['}'],
            || -> ParseResult<Documented<StructField>> {
                let docstring = cursor.consume_docstring()?;
                Ok(StructFieldParser
                    .parse_with(cursor)?
                    .with_doc_comment(docstring))
            }
        );

        cursor.next_token(); // `}`

        Ok(fields)
    }
}

impl Parse for StructItemParser {
    type Output = Item;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.next_token();

        let name = cursor.consume_identifier("struct name")?;

        let generic_parameters = GenericParametersParser.optionally_parse_with(cursor)?;

        let where_clause = WhereClauseParser.optionally_parse_with(cursor)?;

        let fields = StructFieldsParser.parse_with(cursor)?;

        Ok(Item::Struct {
            visibility: self.visibility,
            name,
            generic_parameters,
            where_clause,
            fields,
        })
    }
}

impl Parse for FunctionParameterParser {
    type Output = FunctionParameter;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        let name = cursor.consume_identifier("function parameter name")?;

        cursor.consume(Token![:], "function parameter name")?;

        let r#type = TypeParser.parse_with(cursor)?;

        let mut default_value = None;

        if *cursor.next.unwrap() == Token![=] {
            cursor.next_token();
            default_value = Some(ExpressionParser::default().parse_with(cursor)?);
        }

        Ok(FunctionParameter {
            name,
            r#type,
            default_value,
        })
    }
}

impl Parse for FunctionParser {
    type Output = Function;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
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

        Ok(Function {
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

                    return Err(ParseError::unexpected_token(
                        cursor.current.clone(),
                        expected!(Token![;], Token!['(']),
                        "end of function",
                    ));
                }
            },
        })
    }
}

impl Parse for TraitItemsParser {
    type Output = Vec<Documented<TraitItem>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        let mut items = vec![];

        while *cursor.next.unwrap() != Token!['}'] {
            let doc = cursor.consume_docstring()?;

            let visibility = if *cursor.next.unwrap() == Token![pub] {
                cursor.next_token();
                Visibility::public(cursor.current.span())
            } else {
                Visibility::private()
            };

            items.push(match cursor.next.unwrap() {
                Token![fun] => Ok(TraitItem::AssociatedFunction(
                    FunctionParser { visibility }.parse_with(cursor)?,
                )
                .with_doc_comment(doc)),
                Token![type] => Ok(TraitItem::TypeAlias(
                    TypeAliasParser { visibility }.parse_with(cursor)?,
                )
                .with_doc_comment(doc)),
                _ => Err(ParseError::unexpected_token(
                    cursor.next.clone(),
                    expected!(Token![fun], Token![type]),
                    "trait item",
                )),
            }?);
        }

        Ok(items)
    }
}

impl Parse for TypeAliasParser {
    type Output = TypeAlias;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
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

        Ok(TypeAlias {
            visibility: self.visibility,
            name,
            generic_parameters,
            value,
        })
    }
}

impl Parse for TraitItemParser {
    type Output = Item;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.next_token();

        let name = cursor.consume_identifier("trait name in trait declaration")?;

        let generic_parameters = GenericParametersParser.optionally_parse_with(cursor)?;

        let where_clause = WhereClauseParser.optionally_parse_with(cursor)?;

        cursor.consume(Token!['{'], "trait declaration")?;

        let items = TraitItemsParser.parse_with(cursor)?;

        cursor.consume(Token!['}'], "trait declaration")?;

        Ok(Item::Trait {
            visibility: self.visibility,
            name,
            generic_parameters,
            where_clause,
            items,
        })
    }
}

impl Parse for ImplItemParser {
    type Output = Item;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.next_token();

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

        let items = TraitItemsParser.parse_with(cursor)?;

        cursor.consume(Token!['}'], "type implementation")?;

        Ok(Item::Impl {
            visibility: self.visibility,
            generic_parameters,
            r#type,
            r#trait,
            where_clause,
            items,
        })
    }
}

impl Parse for EnumParser {
    type Output = Item;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.next_token();

        let name = cursor.consume_identifier("enum name")?;

        let generic_parameters = GenericParametersParser.optionally_parse_with(cursor)?;

        cursor.consume(Token!['{'], "enum")?;

        let items = parse_list!(
            cursor,
            "enum items",
            Token!['}'],
            || -> ParseResult<Documented<EnumItem>> {
                let doc = cursor.consume_docstring()?;
                Ok(EnumItemParser.parse_with(cursor)?.with_doc_comment(doc))
            }
        );

        cursor.next_token(); // `}`

        let where_clause = WhereClauseParser.optionally_parse_with(cursor)?;

        Ok(Item::Enum {
            visibility: self.visibility,
            name,
            generic_parameters,
            where_clause,
            items,
        })
    }
}

impl Parse for EnumItemParser {
    type Output = EnumItem;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        let name = cursor.consume_identifier("enum item")?;

        match cursor.next.unwrap() {
            Token!['{'] => EnumItemStructParser { name }.parse_with(cursor),
            Token!['('] => EnumItemTupleParser { name }.parse_with(cursor),
            _ => Ok(EnumItem::Identifier(name)),
        }
    }
}

impl Parse for EnumItemStructParser {
    type Output = EnumItem;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        let fields = StructFieldsParser.parse_with(cursor)?;

        Ok(EnumItem::Struct {
            name: self.name,
            fields,
        })
    }
}

impl Parse for EnumItemTupleParser {
    type Output = EnumItem;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.next_token(); // `(`

        let fields = parse_list!(
            cursor,
            "enum item tuple",
            Token![')'],
            || -> ParseResult<TupleField> {
                let visibility = if cursor.next.unwrap() == &Token![pub] {
                    cursor.next_token();
                    Visibility::public(cursor.current.span())
                } else {
                    Visibility::private()
                };

                let r#type = TypeParser.parse_with(cursor)?;

                Ok(TupleField { visibility, r#type })
            }
        );

        cursor.next_token(); // `)`

        Ok(EnumItem::Tuple {
            name: self.name,
            fields,
        })
    }
}

impl Parse for ItemsParser {
    type Output = Items;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        let mut items = vec![];
        let mut docstring = self.first_docstring;

        while *cursor.next.unwrap() != RawToken::EndOfFile {
            items.push(ItemParser.parse_with(cursor)?.with_doc_comment(docstring));

            docstring = cursor.consume_docstring()?;
        }

        Ok(items)
    }
}

impl Parse for ItemParser {
    type Output = Item;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        let mut visibility = Visibility::private();

        if *cursor.next.unwrap() == Token![pub] {
            visibility = Visibility::public(cursor.next.span());
            cursor.next_token();
        }

        Ok(match cursor.next.unwrap() {
            Token![enum] => EnumParser { visibility }.parse_with(cursor)?,
            Token![import] => ImportParser { visibility }.parse_with(cursor)?,
            Token![struct] => StructItemParser { visibility }.parse_with(cursor)?,
            Token![trait] => TraitItemParser { visibility }.parse_with(cursor)?,
            Token![fun] => Item::Function(FunctionParser { visibility }.parse_with(cursor)?),
            Token![impl] => ImplItemParser { visibility }.parse_with(cursor)?,
            Token![type] => Item::TypeAlias(TypeAliasParser { visibility }.parse_with(cursor)?),
            _ => {
                let error = Err(ParseError::unexpected_token(
                    cursor.next.clone(),
                    expected!(
                        Token![import],
                        Token![fun],
                        Token![trait],
                        Token![enum],
                        Token![struct],
                        Token![impl],
                        Token![type]
                    ),
                    "item",
                ));
                cursor.next_token();
                return error;
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
    parse_test!(ItemParser, enum1, "enum Result[T, E] { Ok(T), Err(E) }");
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
