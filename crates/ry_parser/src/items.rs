use crate::{
    expression::ExpressionParser,
    macros::parse_list,
    path::PathParser,
    r#type::{GenericParametersParser, TypeParser, WhereClauseParser},
    statement::StatementsBlockParser,
    OptionalParser, Parse, TokenIterator,
};
use ry_ast::{
    token::RawToken, Docstring, Documented, EnumItem, Function, FunctionParameter, IdentifierAst,
    Item, ItemKind, Items, StructField, Token, TraitItem, TupleField, TypeAlias, Visibility,
    WithDocComment,
};
use ry_diagnostics::{expected, parser::ParseDiagnostic, Report};
use ry_source_file::span::Span;

struct UseItemParser {
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

struct ItemTupleParser {
    pub(crate) context: ItemKind,
}

struct EnumItemStructParser {
    pub(crate) name: IdentifierAst,
}

pub(crate) struct ItemsParser {
    pub(crate) first_docstring: Docstring,
}

pub(crate) struct ItemParser;

impl Parse for UseItemParser {
    type Output = Option<Item>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        iterator.next_token();

        let path = PathParser.parse_using(iterator)?;
        iterator.consume(Token![;], "import")?;

        Some(Item::Use {
            visibility: self.visibility,
            path,
        })
    }
}

impl Parse for StructFieldParser {
    type Output = Option<StructField>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let visibility = if iterator.next.raw == Token![pub] {
            iterator.next_token();
            Visibility::public(iterator.current.span)
        } else {
            Visibility::private()
        };

        let name = iterator.consume_identifier("struct field")?;

        iterator.consume(Token![:], "struct field")?;

        let r#type = TypeParser.parse_using(iterator)?;

        Some(StructField {
            visibility,
            name,
            r#type,
        })
    }
}

impl Parse for StructFieldsParser {
    type Output = Option<Vec<Documented<StructField>>>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        iterator.consume(Token!['{'], "struct fields")?;

        let fields = parse_list!(
            iterator,
            "struct fields",
            Token!['}'],
            || -> Option<Documented<StructField>> {
                let docstring = iterator.consume_docstring();
                Some(
                    StructFieldParser
                        .parse_using(iterator)?
                        .with_doc_comment(docstring),
                )
            }
        );

        iterator.next_token(); // `}`

        Some(fields)
    }
}

impl Parse for StructItemParser {
    type Output = Option<Item>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        iterator.next_token();

        let name = iterator.consume_identifier("struct name")?;

        let generic_parameters = GenericParametersParser.optionally_parse_using(iterator)?;

        let where_clause = WhereClauseParser.optionally_parse_using(iterator)?;

        if iterator.next.raw == Token!['{'] {
            let fields = StructFieldsParser.parse_using(iterator)?;
            Some(Item::Struct {
                visibility: self.visibility,
                name,
                generic_parameters,
                where_clause,
                fields,
            })
        } else if iterator.next.raw == Token!['('] {
            let fields = ItemTupleParser {
                context: ItemKind::Struct,
            }
            .parse_using(iterator)?;

            if iterator.next.raw == Token![;] {
                iterator.next_token();
            } else {
                iterator.diagnostics.push(
                    ParseDiagnostic::UnexpectedTokenError {
                        got: iterator.current,
                        expected: expected!(Token![;]),
                        node: "struct item".to_owned(),
                    }
                    .build(),
                );
            }

            Some(Item::TupleLikeStruct {
                visibility: self.visibility,
                name,
                generic_parameters,
                where_clause,
                fields,
            })
        } else {
            iterator.diagnostics.push(
                ParseDiagnostic::UnexpectedTokenError {
                    got: iterator.current,
                    expected: expected!(Token![;], Token!['(']),
                    node: "item".to_owned(),
                }
                .build(),
            );

            None
        }
    }
}

impl Parse for FunctionParameterParser {
    type Output = Option<FunctionParameter>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let name = iterator.consume_identifier("function parameter name")?;

        iterator.consume(Token![:], "function parameter name")?;

        let r#type = TypeParser.parse_using(iterator)?;

        let default_value = if iterator.next.raw == Token![=] {
            iterator.next_token();
            Some(ExpressionParser::default().parse_using(iterator)?)
        } else {
            None
        };

        Some(FunctionParameter {
            name,
            r#type,
            default_value,
        })
    }
}

impl Parse for FunctionParser {
    type Output = Option<Function>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        iterator.next_token();

        let name = iterator.consume_identifier("function name")?;

        let generic_parameters = GenericParametersParser.optionally_parse_using(iterator)?;

        iterator.consume(Token!['('], "function")?;

        let parameters = parse_list!(iterator, "function parameters", Token![')'], || {
            FunctionParameterParser.parse_using(iterator)
        });

        iterator.next_token();

        let return_type = if iterator.next.raw == Token![:] {
            iterator.next_token();
            Some(TypeParser.parse_using(iterator)?)
        } else {
            None
        };

        let where_clause = WhereClauseParser.optionally_parse_using(iterator)?;

        Some(Function {
            visibility: self.visibility,
            name,
            generic_parameters,
            parameters,
            return_type,
            where_clause,
            body: match iterator.next.raw {
                Token![;] => {
                    iterator.next_token();

                    None
                }
                Token!['{'] => Some(StatementsBlockParser.parse_using(iterator)?),
                _ => {
                    iterator.next_token();

                    iterator.diagnostics.push(
                        ParseDiagnostic::UnexpectedTokenError {
                            got: iterator.current,
                            expected: expected!(Token![;], Token!['(']),
                            node: "function".to_owned(),
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

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let mut items = vec![];

        while iterator.next.raw != Token!['}'] {
            let doc = iterator.consume_docstring();

            let visibility = if iterator.next.raw == Token![pub] {
                iterator.next_token();
                Visibility::public(iterator.current.span)
            } else {
                Visibility::private()
            };

            items.push(match iterator.next.raw {
                Token![fun] => Some(
                    TraitItem::AssociatedFunction(
                        FunctionParser { visibility }.parse_using(iterator)?,
                    )
                    .with_doc_comment(doc),
                ),
                Token![type] => Some(
                    TraitItem::TypeAlias(TypeAliasParser { visibility }.parse_using(iterator)?)
                        .with_doc_comment(doc),
                ),
                RawToken::EndOfFile => {
                    iterator.diagnostics.push(
                        ParseDiagnostic::EOFInsteadOfCloseBraceForItemError {
                            item_kind: self.item_kind,
                            item_name_span: self.name_span,
                            span: iterator.current.span,
                        }
                        .build(),
                    );
                    return Some((items, true));
                }
                _ => {
                    iterator.diagnostics.push(
                        ParseDiagnostic::UnexpectedTokenError {
                            got: iterator.next,
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

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        iterator.next_token();

        let name = iterator.consume_identifier("type alias")?;
        let generic_parameters = GenericParametersParser.optionally_parse_using(iterator)?;

        let value = if iterator.next.raw == Token![=] {
            iterator.next_token();

            Some(TypeParser.parse_using(iterator)?)
        } else {
            None
        };

        iterator.consume(Token![;], "type alias")?;

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

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        iterator.next_token();

        let name = iterator.consume_identifier("trait name in trait declaration")?;

        let generic_parameters = GenericParametersParser.optionally_parse_using(iterator)?;

        let where_clause = WhereClauseParser.optionally_parse_using(iterator)?;

        iterator.consume(Token!['{'], "trait declaration")?;

        let items = TraitItemsParser {
            name_span: name.span,
            item_kind: ItemKind::Trait,
        }
        .parse_using(iterator)?;

        if !items.1 {
            iterator.consume(Token!['}'], "trait declaration")?;
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

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        iterator.next_token();
        let impl_span = iterator.current.span;

        let generic_parameters = GenericParametersParser.optionally_parse_using(iterator)?;

        let mut r#type = TypeParser.parse_using(iterator)?;
        let mut r#trait = None;

        if iterator.next.raw == Token![for] {
            iterator.next_token();

            r#trait = Some(r#type);
            r#type = TypeParser.parse_using(iterator)?;
        }

        let where_clause = WhereClauseParser.optionally_parse_using(iterator)?;

        iterator.consume(Token!['{'], "type implementation")?;

        let items = TraitItemsParser {
            name_span: impl_span,
            item_kind: ItemKind::Impl,
        }
        .parse_using(iterator)?;

        if !items.1 {
            iterator.consume(Token!['}'], "type implementation")?;
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

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        iterator.next_token();

        let name = iterator.consume_identifier("enum name")?;

        let generic_parameters = GenericParametersParser.optionally_parse_using(iterator)?;

        iterator.consume(Token!['{'], "enum")?;

        let items = parse_list!(
            iterator,
            "enum items",
            Token!['}'],
            || -> Option<Documented<EnumItem>> {
                let doc = iterator.consume_docstring();
                Some(EnumItemParser.parse_using(iterator)?.with_doc_comment(doc))
            }
        );

        iterator.next_token(); // `}`

        let where_clause = WhereClauseParser.optionally_parse_using(iterator)?;

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

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let name = iterator.consume_identifier("enum item")?;

        match iterator.next.raw {
            Token!['{'] => EnumItemStructParser { name }.parse_using(iterator),
            Token!['('] => Some(EnumItem::Tuple {
                name,
                fields: ItemTupleParser {
                    context: ItemKind::Enum,
                }
                .parse_using(iterator)?,
            }),
            _ => Some(EnumItem::Just(name)),
        }
    }
}

impl Parse for EnumItemStructParser {
    type Output = Option<EnumItem>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let fields = StructFieldsParser.parse_using(iterator)?;

        Some(EnumItem::Struct {
            name: self.name,
            fields,
        })
    }
}

impl Parse for ItemTupleParser {
    type Output = Option<Vec<TupleField>>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        iterator.next_token(); // `(`

        let fields = parse_list!(
            iterator,
            format!("item tuple in {}", self.context.to_string()),
            Token![')'],
            || -> Option<TupleField> {
                let visibility = if iterator.next.raw == Token![pub] {
                    iterator.next_token();
                    Visibility::public(iterator.current.span)
                } else {
                    Visibility::private()
                };

                let r#type = TypeParser.parse_using(iterator)?;

                Some(TupleField { visibility, r#type })
            }
        );

        iterator.next_token(); // `)`

        Some(fields)
    }
}

impl Parse for ItemsParser {
    type Output = Items;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let mut items = vec![];
        let mut docstring = self.first_docstring;

        while iterator.next.raw != RawToken::EndOfFile {
            if let Some(item) = ItemParser.parse_using(iterator) {
                items.push(item.with_doc_comment(docstring));
            }

            docstring = iterator.consume_docstring();
        }

        items
    }
}

impl ItemParser {
    fn go_to_next_item(iterator: &mut TokenIterator<'_>) {
        loop {
            match iterator.next.raw {
                Token![enum]
                | Token![use]
                | Token![struct]
                | Token![trait]
                | Token![fun]
                | Token![type]
                | Token![impl]
                | RawToken::EndOfFile => break,
                _ => iterator.next_token(),
            }
        }
    }
}

macro_rules! go_to_next_valid_item {
    ($iter:ident, $item:expr) => {
        if let Some(item) = $item {
            item
        } else {
            Self::go_to_next_item($iter);
            return None;
        }
    };
}

impl Parse for ItemParser {
    type Output = Option<Item>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let mut visibility = Visibility::private();

        if iterator.next.raw == Token![pub] {
            visibility = Visibility::public(iterator.next.span);
            iterator.next_token();
        }

        Some(match iterator.next.raw {
            Token![enum] => {
                go_to_next_valid_item!(iterator, EnumParser { visibility }.parse_using(iterator))
            }
            Token![use] => {
                go_to_next_valid_item!(iterator, UseItemParser { visibility }.parse_using(iterator))
            }
            Token![struct] => {
                go_to_next_valid_item!(
                    iterator,
                    StructItemParser { visibility }.parse_using(iterator)
                )
            }
            Token![trait] => {
                go_to_next_valid_item!(
                    iterator,
                    TraitItemParser { visibility }.parse_using(iterator)
                )
            }
            Token![fun] => Item::Function(go_to_next_valid_item!(
                iterator,
                FunctionParser { visibility }.parse_using(iterator)
            )),
            Token![impl] => {
                go_to_next_valid_item!(
                    iterator,
                    ImplItemParser { visibility }.parse_using(iterator)
                )
            }
            Token![type] => Item::TypeAlias(go_to_next_valid_item!(
                iterator,
                TypeAliasParser { visibility }.parse_using(iterator)
            )),
            _ => {
                iterator.diagnostics.push(
                    ParseDiagnostic::UnexpectedTokenError {
                        got: iterator.next,
                        expected: expected!(
                            Token![use],
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
                    match iterator.next.raw {
                        Token![enum]
                        | Token![use]
                        | Token![struct]
                        | Token![trait]
                        | Token![fun]
                        | Token![type]
                        | Token![impl]
                        | RawToken::EndOfFile => break,
                        _ => iterator.next_token(),
                    }
                }
                return None;
            }
        })
    }
}
