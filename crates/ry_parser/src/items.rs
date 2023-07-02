use crate::{
    expression::ExpressionParser,
    macros::parse_list,
    path::PathParser,
    r#type::{GenericParametersParser, TypeBoundsParser, TypeParser, WhereClauseParser},
    statement::StatementsBlockParser,
    OptionalParser, Parse, ParseState, VisibilityParser,
};
use ry_ast::{
    token::RawToken, Docstring, Documented, EnumItem, Function, FunctionParameter, IdentifierAst,
    Item, ItemKind, Items, JustFunctionParameter, SelfParameter, StructField, Token, TraitItem,
    TupleField, TypeAlias, Visibility, WithDocComment,
};
use ry_diagnostics::parser::ParseDiagnostic::UnnecessaryVisibilityQualifierError;
use ry_diagnostics::parser::UnnecessaryVisibilityQualifierContext;
use ry_diagnostics::{expected, parser::ParseDiagnostic, BuildDiagnostic};
use ry_workspace::span::Span;

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
    pub(crate) item_kind: ItemKind,
    pub(crate) item_span: Span,
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

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let path = PathParser.parse(state)?;
        state.consume(Token![;], "import")?;

        Some(Item::Use {
            visibility: self.visibility,
            path,
        })
    }
}

impl Parse for StructFieldParser {
    type Output = Option<StructField>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let visibility = VisibilityParser.parse(state);

        let name = state.consume_identifier("struct field")?;

        state.consume(Token![:], "struct field")?;

        let ty = TypeParser.parse(state)?;

        Some(StructField {
            visibility,
            name,
            ty,
        })
    }
}

impl Parse for StructFieldsParser {
    type Output = Option<Vec<Documented<StructField>>>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.consume(Token!['{'], "struct fields")?;

        let fields = parse_list!(state, "struct fields", Token!['}'], {
            let docstring = state.consume_docstring();
            Some(StructFieldParser.parse(state)?.with_doc_comment(docstring))
        });

        state.advance(); // `}`

        Some(fields)
    }
}

impl Parse for StructItemParser {
    type Output = Option<Item>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let name = state.consume_identifier("struct name")?;

        let generic_parameters = GenericParametersParser.optionally_parse(state)?;

        let where_clause = WhereClauseParser.optionally_parse(state)?;

        if state.next_token.raw == Token!['{'] {
            let fields = StructFieldsParser.parse(state)?;
            Some(Item::Struct {
                visibility: self.visibility,
                name,
                generic_parameters,
                where_clause,
                fields,
            })
        } else if state.next_token.raw == Token!['('] {
            let fields = ItemTupleParser {
                context: ItemKind::Struct,
            }
            .parse(state)?;

            if state.next_token.raw == Token![;] {
                state.advance();
            } else {
                state.diagnostics.push(
                    ParseDiagnostic::UnexpectedTokenError {
                        got: state.current_token,
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
            state.diagnostics.push(
                ParseDiagnostic::UnexpectedTokenError {
                    got: state.current_token,
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
    type Output = Option<JustFunctionParameter>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let name = state.consume_identifier("function parameter name")?;

        state.consume(Token![:], "function parameter name")?;

        let ty = TypeParser.parse(state)?;

        let default_value = if state.next_token.raw == Token![=] {
            state.advance();
            Some(ExpressionParser::default().parse(state)?)
        } else {
            None
        };

        Some(JustFunctionParameter {
            name,
            ty,
            default_value,
        })
    }
}

impl Parse for FunctionParser {
    type Output = Option<Function>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let name = state.consume_identifier("function name")?;

        let generic_parameters = GenericParametersParser.optionally_parse(state)?;

        state.consume(Token!['('], "function")?;

        let parameters = parse_list!(state, "function parameters", Token![')'], {
            if state.next_token.raw == Token![self] {
                state.advance();

                Some(FunctionParameter::Self_(SelfParameter {
                    self_span: state.current_token.span,
                    ty: if state.next_token.raw == Token![:] {
                        state.advance();

                        Some(TypeParser.parse(state)?)
                    } else {
                        None
                    },
                }))
            } else {
                Some(FunctionParameter::Just(
                    FunctionParameterParser.parse(state)?,
                ))
            }
        });

        state.advance();

        let return_type = if state.next_token.raw == Token![:] {
            state.advance();
            Some(TypeParser.parse(state)?)
        } else {
            None
        };

        let where_clause = WhereClauseParser.optionally_parse(state)?;

        Some(Function {
            visibility: self.visibility,
            name,
            generic_parameters,
            parameters,
            return_type,
            where_clause,
            body: match state.next_token.raw {
                Token![;] => {
                    state.advance();

                    None
                }
                Token!['{'] => Some(StatementsBlockParser.parse(state)?),
                _ => {
                    state.advance();

                    state.diagnostics.push(
                        ParseDiagnostic::UnexpectedTokenError {
                            got: state.current_token,
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

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let mut items = vec![];

        while state.next_token.raw != Token!['}'] {
            let doc = state.consume_docstring();

            if let Some(span) = VisibilityParser.parse(state).span_of_pub() {
                state.diagnostics.push(
                    UnnecessaryVisibilityQualifierError {
                        span,
                        context: UnnecessaryVisibilityQualifierContext::TraitItem {
                            item_span: self.item_span,
                            item_kind: self.item_kind,
                        },
                    }
                    .build(),
                );
            }

            items.push(match state.next_token.raw {
                Token![fun] => Some(
                    TraitItem::AssociatedFunction(
                        FunctionParser {
                            visibility: Visibility::private(),
                        }
                        .parse(state)?,
                    )
                    .with_doc_comment(doc),
                ),
                Token![type] => Some(
                    TraitItem::TypeAlias(
                        TypeAliasParser {
                            visibility: Visibility::private(),
                        }
                        .parse(state)?,
                    )
                    .with_doc_comment(doc),
                ),
                _ => {
                    state.diagnostics.push(
                        ParseDiagnostic::UnexpectedTokenError {
                            got: state.next_token,
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

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let name = state.consume_identifier("type alias")?;
        let generic_parameters = GenericParametersParser.optionally_parse(state)?;

        let bounds = if state.next_token.raw == Token![:] {
            state.advance();

            Some(TypeBoundsParser.parse(state)?)
        } else {
            None
        };

        let value = if state.next_token.raw == Token![=] {
            state.advance();

            Some(TypeParser.parse(state)?)
        } else {
            None
        };

        state.consume(Token![;], "type alias")?;

        Some(TypeAlias {
            visibility: self.visibility,
            name,
            generic_parameters,
            bounds,
            value,
        })
    }
}

impl Parse for TraitItemParser {
    type Output = Option<Item>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let name = state.consume_identifier("trait name in trait declaration")?;

        let generic_parameters = GenericParametersParser.optionally_parse(state)?;

        let where_clause = WhereClauseParser.optionally_parse(state)?;

        state.consume(Token!['{'], "trait declaration")?;

        let items = TraitItemsParser {
            item_kind: ItemKind::Trait,
            item_span: name.span,
        }
        .parse(state)?;

        if !items.1 {
            state.consume(Token!['}'], "trait declaration")?;
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

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();
        let impl_span = state.current_token.span;

        if let Some(span) = self.visibility.span_of_pub() {
            state.diagnostics.push(
                UnnecessaryVisibilityQualifierError {
                    span,
                    context: UnnecessaryVisibilityQualifierContext::Impl,
                }
                .build(),
            );
        }

        let generic_parameters = GenericParametersParser.optionally_parse(state)?;

        let mut ty = TypeParser.parse(state)?;
        let mut r#trait = None;

        if state.next_token.raw == Token![for] {
            state.advance();

            r#trait = Some(ty);
            ty = TypeParser.parse(state)?;
        }

        let where_clause = WhereClauseParser.optionally_parse(state)?;

        state.consume(Token!['{'], "type implementation")?;

        let items = TraitItemsParser {
            item_kind: ItemKind::Impl,
            item_span: impl_span,
        }
        .parse(state)?;

        if !items.1 {
            state.consume(Token!['}'], "type implementation")?;
        }

        Some(Item::Impl {
            visibility: self.visibility,
            generic_parameters,
            ty,
            r#trait,
            where_clause,
            items: items.0,
        })
    }
}

impl Parse for EnumParser {
    type Output = Option<Item>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let name = state.consume_identifier("enum name")?;

        let generic_parameters = GenericParametersParser.optionally_parse(state)?;

        state.consume(Token!['{'], "enum")?;

        let items = parse_list!(state, "enum items", Token!['}'], {
            let doc = state.consume_docstring();
            Some(EnumItemParser.parse(state)?.with_doc_comment(doc))
        });

        state.advance(); // `}`

        let where_clause = WhereClauseParser.optionally_parse(state)?;

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

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let name = state.consume_identifier("enum item")?;

        match state.next_token.raw {
            Token!['{'] => EnumItemStructParser { name }.parse(state),
            Token!['('] => Some(EnumItem::Tuple {
                name,
                fields: ItemTupleParser {
                    context: ItemKind::Enum,
                }
                .parse(state)?,
            }),
            _ => Some(EnumItem::Just(name)),
        }
    }
}

impl Parse for EnumItemStructParser {
    type Output = Option<EnumItem>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let fields = StructFieldsParser.parse(state)?;

        Some(EnumItem::Struct {
            name: self.name,
            fields,
        })
    }
}

impl Parse for ItemTupleParser {
    type Output = Option<Vec<TupleField>>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance(); // `(`

        let fields = parse_list!(
            state,
            format!("item tuple in {}", self.context.to_string()),
            Token![')'],
            {
                Some(TupleField {
                    visibility: VisibilityParser.parse(state),
                    ty: TypeParser.parse(state)?,
                })
            }
        );

        state.advance(); // `)`

        Some(fields)
    }
}

impl Parse for ItemsParser {
    type Output = Items;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let mut items = vec![];
        let mut docstring = self.first_docstring;

        while state.next_token.raw != RawToken::EndOfFile {
            if let Some(item) = ItemParser.parse(state) {
                items.push(item.with_doc_comment(docstring));
            }

            docstring = state.consume_docstring();
        }

        items
    }
}

impl ItemParser {
    fn go_to_next_item(state: &mut ParseState<'_, '_, '_>) {
        loop {
            match state.next_token.raw {
                Token![enum]
                | Token![use]
                | Token![struct]
                | Token![trait]
                | Token![fun]
                | Token![type]
                | Token![impl]
                | RawToken::EndOfFile => break,
                _ => state.advance(),
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

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let visibility = VisibilityParser.parse(state);

        Some(match state.next_token.raw {
            Token![enum] => {
                go_to_next_valid_item!(state, EnumParser { visibility }.parse(state))
            }
            Token![use] => {
                go_to_next_valid_item!(state, UseItemParser { visibility }.parse(state))
            }
            Token![struct] => {
                go_to_next_valid_item!(state, StructItemParser { visibility }.parse(state))
            }
            Token![trait] => {
                go_to_next_valid_item!(state, TraitItemParser { visibility }.parse(state))
            }
            Token![fun] => Item::Function(go_to_next_valid_item!(
                state,
                FunctionParser { visibility }.parse(state)
            )),
            Token![impl] => {
                go_to_next_valid_item!(state, ImplItemParser { visibility }.parse(state))
            }
            Token![type] => Item::TypeAlias(go_to_next_valid_item!(
                state,
                TypeAliasParser { visibility }.parse(state)
            )),
            _ => {
                state.diagnostics.push(
                    ParseDiagnostic::UnexpectedTokenError {
                        got: state.next_token,
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
                    match state.next_token.raw {
                        Token![enum]
                        | Token![use]
                        | Token![struct]
                        | Token![trait]
                        | Token![fun]
                        | Token![type]
                        | Token![impl]
                        | RawToken::EndOfFile => break,
                        _ => state.advance(),
                    }
                }
                return None;
            }
        })
    }
}
