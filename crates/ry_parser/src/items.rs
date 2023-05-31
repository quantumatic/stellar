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
    token::RawToken, Docstring, Documented, Function, FunctionParameter, Identifier, Item, Items,
    StructMember, Token, TraitItem, TypeAlias, Visibility, WithDocComment,
};

struct ImportParser {
    pub(crate) visibility: Visibility,
}

struct StructItemParser {
    pub(crate) visibility: Visibility,
}

struct StructMembersParser;

struct StructMemberParser;

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

struct EnumDeclarationParser {
    pub(crate) visibility: Visibility,
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

impl Parse for StructMemberParser {
    type Output = StructMember;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        let mut visibility = Visibility::private();

        if *cursor.next.unwrap() == Token![pub] {
            cursor.next_token();
            visibility = Visibility::public(cursor.current.span());
        }

        let name = cursor.consume_identifier("struct member name in struct definition")?;

        cursor.consume(Token![:], "struct member definition")?;

        let r#type = TypeParser.parse_with(cursor)?;

        cursor.consume(Token![;], "struct member definition")?;

        Ok(StructMember {
            visibility,
            name,
            r#type,
        })
    }
}

impl Parse for StructMembersParser {
    type Output = Vec<Documented<StructMember>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        let mut members = vec![];

        while *cursor.next.unwrap() != Token!['}'] {
            let docstring = cursor.consume_docstring()?;

            members.push(
                StructMemberParser
                    .parse_with(cursor)?
                    .with_doc_comment(docstring),
            );
        }

        Ok(members)
    }
}

impl Parse for StructItemParser {
    type Output = Item;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.next_token();

        let name = cursor.consume_identifier("struct name in struct declaration")?;

        let generic_parameters = GenericParametersParser.optionally_parse_with(cursor)?;

        let where_clause = WhereClauseParser.optionally_parse_with(cursor)?;

        cursor.next_token();

        let members = StructMembersParser.parse_with(cursor)?;

        cursor.consume(Token!['}'], "struct declaration")?;

        Ok(Item::Struct {
            visibility: self.visibility,
            name,
            generic_parameters,
            where_clause,
            members,
        })
    }
}

impl Parse for FunctionParameterParser {
    type Output = FunctionParameter;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        let name = cursor.consume_identifier("function argument name")?;

        cursor.consume(Token![:], "function argument name")?;

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

        let name = cursor.consume_identifier("function name in function declaration")?;

        let generic_parameters = GenericParametersParser.optionally_parse_with(cursor)?;

        cursor.consume(Token!['('], "function declaration")?;

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

impl Parse for EnumDeclarationParser {
    type Output = Item;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.next_token();

        let name = cursor.consume_identifier("enum name in enum declaration")?;

        cursor.consume(Token!['{'], "enum declaration")?;

        let variants = parse_list!(
            cursor,
            "enum declaration",
            Token!['}'],
            || -> ParseResult<Documented<Identifier>> {
                let doc = cursor.consume_docstring()?;
                Ok(cursor
                    .consume_identifier("enum variant name")?
                    .with_doc_comment(doc))
            }
        );

        cursor.next_token();

        Ok(Item::Enum {
            visibility: self.visibility,
            name,
            variants,
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
            Token![enum] => EnumDeclarationParser { visibility }.parse_with(cursor)?,
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
        "struct Point[T: Numeric] { pub x: T; pub y: T; }"
    );
    parse_test!(
        ItemParser,
        r#struct2,
        "struct Lexer[S] where S: Iterator[char] { contents: S; }"
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
    parse_test!(ItemParser, single_variant, "enum test { a }");
    parse_test!(ItemParser, variants, "enum test { a, b, c, }");
}
