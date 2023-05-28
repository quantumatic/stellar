use crate::{
    error::{expected, ParseError, ParseResult},
    expression::ExpressionParser,
    macros::parse_list,
    path::PathParser,
    r#type::{GenericParametersParser, TypeParser, WhereClauseParser},
    statement::StatementsBlockParser,
    OptionalParser, Parser, ParserState,
};
use ry_ast::{
    token::RawToken, Docstring, Documented, Function, FunctionParameter, Identifier, Item, Items,
    StructMember, Token, TraitItem, TypeAlias, Visibility, WithDocComment,
};

#[derive(Default)]
pub(crate) struct ImportParser {
    pub(crate) visibility: Visibility,
}

impl Parser for ImportParser {
    type Output = Item;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();

        let path = PathParser.parse_with(state)?;
        state.consume(Token![;], "import")?;

        Ok(Item::Import {
            visibility: self.visibility,
            path,
        })
    }
}

pub(crate) struct StructMemberParser;

impl Parser for StructMemberParser {
    type Output = StructMember;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let mut visibility = Visibility::private();

        if *state.next.unwrap() == Token![pub] {
            state.next_token();
            visibility = Visibility::public(state.current.span());
        }

        let name = state.consume_identifier("struct member name in struct definition")?;

        state.consume(Token![:], "struct member definition")?;

        let r#type = TypeParser.parse_with(state)?;

        state.consume(Token![;], "struct member definition")?;

        Ok(StructMember {
            visibility,
            name,
            r#type,
        })
    }
}

pub(crate) struct StructMembersParser;

impl Parser for StructMembersParser {
    type Output = Vec<Documented<StructMember>>;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let mut members = vec![];

        while *state.next.unwrap() != Token!['}'] {
            let docstring = state.consume_docstring()?;

            members.push(
                StructMemberParser
                    .parse_with(state)?
                    .with_doc_comment(docstring),
            );
        }

        Ok(members)
    }
}

#[derive(Default)]
pub(crate) struct StructItemParser {
    pub(crate) visibility: Visibility,
}

impl Parser for StructItemParser {
    type Output = Item;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();

        let name = state.consume_identifier("struct name in struct declaration")?;

        let generic_parameters = GenericParametersParser.optionally_parse_with(state)?;

        let where_clause = WhereClauseParser.optionally_parse_with(state)?;

        state.next_token();

        let members = StructMembersParser.parse_with(state)?;

        state.consume(Token!['}'], "struct declaration")?;

        Ok(Item::Struct {
            visibility: self.visibility,
            name,
            generic_parameters,
            where_clause,
            members,
        })
    }
}

pub(crate) struct FunctionParameterParser;

impl Parser for FunctionParameterParser {
    type Output = FunctionParameter;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let name = state.consume_identifier("function argument name")?;

        state.consume(Token![:], "function argument name")?;

        let r#type = TypeParser.parse_with(state)?;

        let mut default_value = None;

        if *state.next.unwrap() == Token![=] {
            state.next_token();
            default_value = Some(ExpressionParser::default().parse_with(state)?);
        }

        Ok(FunctionParameter {
            name,
            r#type,
            default_value,
        })
    }
}

#[derive(Default)]
pub(crate) struct FunctionParser {
    pub(crate) visibility: Visibility,
}

impl Parser for FunctionParser {
    type Output = Function;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();

        let name = state.consume_identifier("function name in function declaration")?;

        let generic_parameters = GenericParametersParser.optionally_parse_with(state)?;

        state.consume(Token!['('], "function declaration")?;

        let parameters = parse_list!(state, "function parameters", Token![')'], || {
            FunctionParameterParser.parse_with(state)
        });

        state.next_token();

        let mut return_type = None;

        if *state.next.unwrap() == Token![:] {
            state.next_token();
            return_type = Some(TypeParser.parse_with(state)?);
        }

        let where_clause = WhereClauseParser.optionally_parse_with(state)?;

        Ok(Function {
            visibility: self.visibility,
            name,
            generic_parameters,
            parameters,
            return_type,
            where_clause,
            body: match state.next.unwrap() {
                Token![;] => {
                    state.next_token();

                    None
                }
                Token!['{'] => Some(StatementsBlockParser.parse_with(state)?),
                _ => {
                    state.next_token();

                    return Err(ParseError::unexpected_token(
                        state.current.clone(),
                        expected!(Token![;], Token!['(']),
                        "end of function",
                    ));
                }
            },
        })
    }
}

pub(crate) struct TraitItemsParser;

impl Parser for TraitItemsParser {
    type Output = Vec<Documented<TraitItem>>;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let mut items = vec![];

        while *state.next.unwrap() != Token!['}'] {
            let doc = state.consume_docstring()?;

            let visibility = if *state.next.unwrap() == Token![pub] {
                state.next_token();
                Visibility::public(state.current.span())
            } else {
                Visibility::private()
            };

            items.push(match state.next.unwrap() {
                Token![fun] => Ok(TraitItem::AssociatedFunction(
                    FunctionParser { visibility }.parse_with(state)?,
                )
                .with_doc_comment(doc)),
                Token![type] => Ok(TraitItem::TypeAlias(
                    TypeAliasParser { visibility }.parse_with(state)?,
                )
                .with_doc_comment(doc)),
                _ => Err(ParseError::unexpected_token(
                    state.next.clone(),
                    expected!(Token![fun], Token![type]),
                    "trait item",
                )),
            }?);
        }

        Ok(items)
    }
}

#[derive(Default)]
pub(crate) struct TypeAliasParser {
    pub(crate) visibility: Visibility,
}

impl Parser for TypeAliasParser {
    type Output = TypeAlias;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();

        let name = state.consume_identifier("type alias")?;
        let generic_parameters = GenericParametersParser.optionally_parse_with(state)?;

        let value = if *state.next.unwrap() == Token![=] {
            state.next_token();

            Some(TypeParser.parse_with(state)?)
        } else {
            None
        };

        state.consume(Token![;], "type alias")?;

        Ok(TypeAlias {
            visibility: self.visibility,
            name,
            generic_parameters,
            value,
        })
    }
}

#[derive(Default)]
pub(crate) struct TraitItemParser {
    pub(crate) visibility: Visibility,
}

impl Parser for TraitItemParser {
    type Output = Item;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();

        let name = state.consume_identifier("trait name in trait declaration")?;

        let generic_parameters = GenericParametersParser.optionally_parse_with(state)?;

        let where_clause = WhereClauseParser.optionally_parse_with(state)?;

        state.consume(Token!['{'], "trait declaration")?;

        let items = TraitItemsParser.parse_with(state)?;

        state.consume(Token!['}'], "trait declaration")?;

        Ok(Item::Trait {
            visibility: self.visibility,
            name,
            generic_parameters,
            where_clause,
            items,
        })
    }
}

#[derive(Default)]
pub(crate) struct ImplItemParser {
    pub(crate) visibility: Visibility,
}

impl Parser for ImplItemParser {
    type Output = Item;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();

        let generic_parameters = GenericParametersParser.optionally_parse_with(state)?;

        let mut r#type = TypeParser.parse_with(state)?;
        let mut r#trait = None;

        if *state.next.unwrap() == Token![for] {
            state.next_token();

            r#trait = Some(r#type);
            r#type = TypeParser.parse_with(state)?;
        }

        let where_clause = WhereClauseParser.optionally_parse_with(state)?;

        state.consume(Token!['{'], "type implementation")?;

        let items = TraitItemsParser.parse_with(state)?;

        state.consume(Token!['}'], "type implementation")?;

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

#[derive(Default)]
pub(crate) struct EnumDeclarationParser {
    pub(crate) visibility: Visibility,
}

impl Parser for EnumDeclarationParser {
    type Output = Item;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();

        let name = state.consume_identifier("enum name in enum declaration")?;

        state.consume(Token!['{'], "enum declaration")?;

        let variants = parse_list!(
            state,
            "enum declaration",
            Token!['}'],
            || -> ParseResult<Documented<Identifier>> {
                let doc = state.consume_docstring()?;
                Ok(state
                    .consume_identifier("enum variant name")?
                    .with_doc_comment(doc))
            }
        );

        state.next_token();

        Ok(Item::Enum {
            visibility: self.visibility,
            name,
            variants,
        })
    }
}

pub(crate) struct ItemsParser {
    pub(crate) first_docstring: Docstring,
}

impl Parser for ItemsParser {
    type Output = Items;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let mut items = vec![];
        let mut docstring = self.first_docstring;

        while *state.next.unwrap() != RawToken::EndOfFile {
            items.push(ItemParser.parse_with(state)?.with_doc_comment(docstring));

            docstring = state.consume_docstring()?;
        }

        Ok(items)
    }
}

pub(crate) struct ItemParser;

impl Parser for ItemParser {
    type Output = Item;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let mut visibility = Visibility::private();

        if *state.next.unwrap() == Token![pub] {
            visibility = Visibility::public(state.next.span());
            state.next_token();
        }

        Ok(match state.next.unwrap() {
            Token![enum] => EnumDeclarationParser { visibility }.parse_with(state)?,
            Token![import] => ImportParser { visibility }.parse_with(state)?,
            Token![struct] => StructItemParser { visibility }.parse_with(state)?,
            Token![trait] => TraitItemParser { visibility }.parse_with(state)?,
            Token![fun] => Item::Function(FunctionParser { visibility }.parse_with(state)?),
            Token![impl] => ImplItemParser { visibility }.parse_with(state)?,
            Token![type] => Item::TypeAlias(TypeAliasParser { visibility }.parse_with(state)?),
            _ => {
                let error = Err(ParseError::unexpected_token(
                    state.next.clone(),
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
                state.next_token();
                return error;
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::macros::parser_test;

    parser_test!(FunctionParser, function1, "fun test();");
    parser_test!(FunctionParser, function2, "fun test[A](a: A): A { a }");
    parser_test!(
        FunctionParser,
        function3,
        "fun unwrap[T, B: Option[T]](a: B): T { a.unwrap() }"
    );
    parser_test!(ImplItemParser, impl1, "impl[T] NotOption for T {}");
    parser_test!(
        ImplItemParser,
        impl2,
        "impl[T] Into[Option[M]] for Tuple[T, M] where M: Into[T] {}"
    );
    parser_test!(ImportParser, single_import, "import test;");
    parser_test!(ImportParser, imports, "import test; import test2.test;");
    parser_test!(StructItemParser, empty_struct, "struct test {}");
    parser_test!(
        StructItemParser,
        r#struct1,
        "struct Point[T: Numeric] { pub x: T; pub y: T; }"
    );
    parser_test!(
        StructItemParser,
        r#struct2,
        "struct Lexer[S] where S: Iterator[char] { contents: S; }"
    );
    parser_test!(TraitItemParser, empty_trait, "trait test {}");
    parser_test!(TraitItemParser, trait1, "trait test { fun f(); }");
    parser_test!(
        TraitItemParser,
        trait2,
        "trait Into[T] { fun into(self: Self): T; }"
    );
    parser_test!(TypeAliasParser, empty_type_alias, "type A;");
    parser_test!(TypeAliasParser, type_alias1, "type B = Option[i32];");
    parser_test!(TypeAliasParser, type_alias2, "type B[T] = Option[T];");
    parser_test!(EnumDeclarationParser, no_variants, "enum test {}");
    parser_test!(EnumDeclarationParser, single_variant, "enum test { a }");
    parser_test!(EnumDeclarationParser, variants, "enum test { a, b, c, }");
}
