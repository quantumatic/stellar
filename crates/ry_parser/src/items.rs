use ry_ast::{
    token::RawToken, EnumItem, Function, FunctionParameter, FunctionSignature, IdentifierAST,
    ModuleItem, ModuleItemKind, NotSelfFunctionParameter, SelfFunctionParameter, StructField,
    Token, TupleField, TypeAlias, Visibility,
};
use ry_interner::builtin_symbols;

use crate::{
    diagnostics::{
        UnexpectedTokenDiagnostic, UnnecessaryVisibilityQualifierContext,
        UnnecessaryVisibilityQualifierDiagnostic,
    },
    expected,
    macros::parse_list,
    path::ImportPathParser,
    r#type::{BoundsParser, GenericParametersParser, TypeParser, WherePredicatesParser},
    statement::StatementsBlockParser,
    OptionallyParse, Parse, ParseState, VisibilityParser,
};

struct ImportParser {
    pub(crate) visibility: Visibility,
}

struct StructParser {
    pub(crate) visibility: Visibility,
    pub(crate) docstring: Option<String>,
}

struct StructFieldsParser;

struct StructFieldParser {
    pub(crate) visibility: Visibility,
    pub(crate) docstring: Option<String>,
}

struct FunctionParser {
    pub(crate) visibility: Visibility,
    pub(crate) docstring: Option<String>,
}

pub(crate) struct NotSelfFunctionParameterParser;

struct TypeAliasParser {
    pub(crate) visibility: Visibility,
    pub(crate) docstring: Option<String>,
}

struct InterfaceParser {
    pub(crate) visibility: Visibility,
    pub(crate) docstring: Option<String>,
}

struct EnumParser {
    pub(crate) visibility: Visibility,
    pub(crate) docstring: Option<String>,
}

struct EnumItemParser;

struct TupleFieldsParser {
    pub(crate) context: ModuleItemKind,
}

struct EnumItemStructParser {
    pub(crate) name: IdentifierAST,
    pub(crate) docstring: Option<String>,
}

pub(crate) struct ItemParser;

pub(crate) struct ItemsParser;

impl Parse for ImportParser {
    type Output = Option<ModuleItem>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.location.start;

        if let Visibility::Public(location) = self.visibility {
            state.add_diagnostic(UnnecessaryVisibilityQualifierDiagnostic {
                location,
                context: UnnecessaryVisibilityQualifierContext::Import,
            });
        }

        state.advance();

        let path = ImportPathParser.parse(state)?;
        state.consume(Token![;], "import")?;

        Some(ModuleItem::Import {
            path,
            location: state.location_from(start),
        })
    }
}

impl Parse for StructFieldParser {
    type Output = Option<StructField>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let name = state.consume_identifier("struct field")?;

        state.consume(Token![:], "struct field")?;

        let ty = TypeParser.parse(state)?;

        Some(StructField {
            visibility: self.visibility,
            name,
            ty,
            docstring: self.docstring,
        })
    }
}

impl Parse for StructFieldsParser {
    type Output = Option<Vec<StructField>>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.consume(Token!['{'], "struct fields")?;

        let fields = parse_list!(state, "struct fields", Token!['}'], {
            Some(
                StructFieldParser {
                    docstring: state.consume_local_docstring(),
                    visibility: VisibilityParser.parse(state),
                }
                .parse(state)?,
            )
        });

        state.advance(); // `}`

        Some(fields)
    }
}

impl Parse for StructParser {
    type Output = Option<ModuleItem>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let name = state.consume_identifier("struct name")?;

        let generic_parameters = GenericParametersParser.optionally_parse(state)?;

        if state.next_token.raw == Token!['('] {
            let fields = TupleFieldsParser {
                context: ModuleItemKind::Struct,
            }
            .parse(state)?;

            let implements = if state.next_token.raw == Token![implements] {
                state.advance();

                Some(BoundsParser.parse(state)?)
            } else {
                None
            };

            let where_predicates = WherePredicatesParser.optionally_parse(state)?;

            let mut methods = vec![];

            if state.next_token.raw != Token![;] {
                state.consume(Token!['{'], "struct")?;

                loop {
                    if state.next_token.raw == Token!['}'] {
                        break;
                    }

                    methods.push(
                        FunctionParser {
                            visibility: VisibilityParser.parse(state),
                            docstring: state.consume_local_docstring(),
                        }
                        .parse(state)?,
                    );
                }

                state.advance();
            }

            state.advance();

            Some(ModuleItem::TupleLikeStruct {
                visibility: self.visibility,
                name,
                generic_parameters,
                where_predicates,
                fields,
                methods,
                implements,
                docstring: self.docstring,
            })
        } else if state.next_token.raw == Token!['{'] {
            let implements = if state.next_token.raw == Token![implements] {
                state.advance();

                Some(BoundsParser.parse(state)?)
            } else {
                None
            };

            let where_predicates = WherePredicatesParser.optionally_parse(state)?;

            let fields = parse_list!(state, "struct fields", (Token!['}']) or (Token![fun]), {
                let docstring = state.consume_local_docstring();
                let visibility = VisibilityParser.parse(state);

                StructFieldParser {
                    visibility,
                    docstring,
                }
                .parse(state)
            });

            let mut methods = vec![];

            if state.next_token.raw == Token![fun] {
                loop {
                    if state.next_token.raw == Token!['}'] {
                        break;
                    }

                    let docstring = state.consume_local_docstring();
                    let visibility = VisibilityParser.parse(state);

                    methods.push(
                        FunctionParser {
                            visibility,
                            docstring,
                        }
                        .parse(state)?,
                    );
                }
            }

            Some(ModuleItem::Struct {
                visibility: self.visibility,
                name,
                generic_parameters,
                where_predicates,
                fields,
                methods,
                implements,
                docstring: self.docstring,
            })
        } else {
            state.add_diagnostic(UnexpectedTokenDiagnostic::new(
                state.current_token,
                expected!(Token![;], Token!['(']),
                "item",
            ));

            None
        }
    }
}

impl Parse for NotSelfFunctionParameterParser {
    type Output = Option<NotSelfFunctionParameter>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let name = state.consume_identifier("function parameter name")?;

        state.consume(Token![:], "function parameter name")?;

        let ty = TypeParser.parse(state)?;

        Some(NotSelfFunctionParameter { name, ty })
    }
}

impl Parse for FunctionParser {
    type Output = Option<Function>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.consume(Token![fun], "function")?;

        let name = state.consume_identifier("function name")?;

        let generic_parameters = GenericParametersParser.optionally_parse(state)?;

        state.consume(Token!['('], "function")?;

        let parameters = parse_list!(state, "function parameters", Token![')'], {
            if state.lexer.scanned_identifier == builtin_symbols::SMALL_SELF {
                state.advance();

                Some(FunctionParameter::SelfParameter(SelfFunctionParameter {
                    self_location: state.current_token.location,
                    ty: if state.next_token.raw == Token![:] {
                        state.advance();

                        Some(TypeParser.parse(state)?)
                    } else {
                        None
                    },
                }))
            } else {
                Some(FunctionParameter::NotSelfParameter(
                    NotSelfFunctionParameterParser.parse(state)?,
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

        let where_predicates = WherePredicatesParser.optionally_parse(state)?;

        Some(Function {
            signature: FunctionSignature {
                visibility: self.visibility,
                name,
                generic_parameters,
                parameters,
                return_type,
                where_predicates,
                docstring: self.docstring,
            },
            body: match state.next_token.raw {
                Token![;] => {
                    state.advance();

                    None
                }
                Token!['{'] => Some(StatementsBlockParser.parse(state)?),
                _ => {
                    state.advance();

                    state.add_diagnostic(UnexpectedTokenDiagnostic::new(
                        state.current_token,
                        expected!(Token![;], Token!['(']),
                        "function",
                    ));

                    None
                }
            },
        })
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

            Some(BoundsParser.parse(state)?)
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
            docstring: self.docstring,
        })
    }
}

impl Parse for InterfaceParser {
    type Output = Option<ModuleItem>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let name = state.consume_identifier("interface name in interface declaration")?;

        let generic_parameters = GenericParametersParser.optionally_parse(state)?;

        let implements = if state.next_token.raw == Token![implements] {
            state.advance();

            Some(BoundsParser.parse(state)?)
        } else {
            None
        };

        let where_predicates = WherePredicatesParser.optionally_parse(state)?;

        state.consume(Token!['{'], "interface declaration")?;

        let methods = parse_list!(state, "interface methods", Token!['}'], {
            FunctionParser {
                docstring: state.consume_local_docstring(),
                visibility: VisibilityParser.parse(state),
            }
            .parse(state)
        });

        state.advance();

        Some(ModuleItem::Interface {
            visibility: self.visibility,
            name,
            generic_parameters,
            where_predicates,
            methods,
            implements,
            docstring: self.docstring,
        })
    }
}

impl Parse for EnumParser {
    type Output = Option<ModuleItem>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let name = state.consume_identifier("enum name")?;

        let generic_parameters = GenericParametersParser.optionally_parse(state)?;

        let implements = if state.next_token.raw == Token![implements] {
            state.advance();

            Some(BoundsParser.parse(state)?)
        } else {
            None
        };

        let where_predicates = WherePredicatesParser.optionally_parse(state)?;

        state.consume(Token!['{'], "enum")?;

        let items = parse_list!(state, "enum items", (Token!['}']) or (Token![fun]), {
            EnumItemParser.parse(state)
        });

        let mut methods = vec![];

        loop {
            if state.next_token.raw == Token!['}'] {
                break;
            }

            let docstring = state.consume_local_docstring();
            let visibility = VisibilityParser.parse(state);

            methods.push(
                FunctionParser {
                    visibility,
                    docstring,
                }
                .parse(state)?,
            );
        }

        state.advance(); // `}`

        Some(ModuleItem::Enum {
            visibility: self.visibility,
            name,
            generic_parameters,
            where_predicates,
            items,
            methods,
            implements,
            docstring: self.docstring,
        })
    }
}

impl Parse for EnumItemParser {
    type Output = Option<EnumItem>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let docstring = state.consume_local_docstring();

        let name = state.consume_identifier("enum item")?;

        match state.next_token.raw {
            Token!['{'] => EnumItemStructParser { name, docstring }.parse(state),
            Token!['('] => Some(EnumItem::TupleLike {
                name,
                fields: TupleFieldsParser {
                    context: ModuleItemKind::Enum,
                }
                .parse(state)?,
                docstring,
            }),
            _ => Some(EnumItem::Just { name, docstring }),
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
            docstring: self.docstring,
        })
    }
}

impl Parse for TupleFieldsParser {
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
    type Output = Vec<ModuleItem>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let mut items = vec![];

        while state.next_token.raw != RawToken::EndOfFile {
            if let Some(item) = ItemParser.parse(state) {
                items.push(item);
            }
        }

        items
    }
}

impl ItemParser {
    fn goto_next_valid_item(state: &mut ParseState<'_, '_, '_>) {
        loop {
            match state.next_token.raw {
                Token![enum]
                | Token![import]
                | Token![struct]
                | Token![fun]
                | Token![type]
                | Token![interface]
                | RawToken::EndOfFile => break,
                _ => state.advance(),
            }
        }
    }
}

macro_rules! possibly_recover {
    ($state:ident, $item:expr) => {
        if let Some(item) = $item {
            item
        } else {
            Self::goto_next_valid_item($state);
            return None;
        }
    };
}

impl Parse for ItemParser {
    type Output = Option<ModuleItem>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let docstring = state.consume_local_docstring();
        let visibility = VisibilityParser.parse(state);

        Some(match state.next_token.raw {
            Token![enum] => {
                possibly_recover!(
                    state,
                    EnumParser {
                        visibility,
                        docstring
                    }
                    .parse(state)
                )
            }
            Token![import] => {
                possibly_recover!(state, ImportParser { visibility }.parse(state))
            }
            Token![struct] => {
                possibly_recover!(
                    state,
                    StructParser {
                        visibility,
                        docstring
                    }
                    .parse(state)
                )
            }
            Token![interface] => {
                possibly_recover!(
                    state,
                    InterfaceParser {
                        visibility,
                        docstring
                    }
                    .parse(state)
                )
            }
            Token![fun] => ModuleItem::Function(possibly_recover!(
                state,
                FunctionParser {
                    visibility,
                    docstring
                }
                .parse(state)
            )),
            Token![type] => ModuleItem::TypeAlias(possibly_recover!(
                state,
                TypeAliasParser {
                    visibility,
                    docstring
                }
                .parse(state)
            )),
            _ => {
                state.add_diagnostic(UnexpectedTokenDiagnostic::new(
                    state.next_token,
                    expected!(
                        Token![import],
                        Token![fun],
                        Token![interface],
                        Token![enum],
                        Token![struct],
                        Token![type]
                    ),
                    "item",
                ));

                Self::goto_next_valid_item(state);
                return None;
            }
        })
    }
}
