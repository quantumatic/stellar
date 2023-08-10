use ry_ast::{
    token::{Keyword, Punctuator, RawToken},
    EnumItem, Function, FunctionParameter, FunctionSignature, IdentifierAST, ModuleItem,
    ModuleItemKind, NotSelfFunctionParameter, SelfFunctionParameter, StructField, TupleField,
    TypeAlias, Visibility,
};
use ry_interner::builtin_identifiers;

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
    visibility: Visibility,
}

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

        state.consume(Punctuator::Semicolon, "import")?;

        Some(ModuleItem::Import {
            path,
            location: state.location_from(start),
        })
    }
}

struct StructFieldParser {
    pub(crate) visibility: Visibility,
    pub(crate) docstring: Option<String>,
}

impl Parse for StructFieldParser {
    type Output = Option<StructField>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let name = state.consume_identifier("struct field")?;

        state.consume(Punctuator::Colon, "struct field")?;

        let ty = TypeParser.parse(state)?;

        Some(StructField {
            visibility: self.visibility,
            name,
            ty,
            docstring: self.docstring,
        })
    }
}

struct StructFieldsParser;

impl Parse for StructFieldsParser {
    type Output = Option<Vec<StructField>>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.consume(Punctuator::OpenBrace, "struct fields")?;

        let fields = parse_list!(
            state,
            node_name: "struct fields",
            closing_token: Punctuator::CloseBrace,
            parse_element_expr:
                StructFieldParser {
                    docstring: state.consume_local_docstring(),
                    visibility: VisibilityParser.parse(state),
                }
                .parse(state)
        );

        state.advance(); // `}`

        Some(fields)
    }
}

struct StructParser {
    pub(crate) visibility: Visibility,
    pub(crate) docstring: Option<String>,
}

impl Parse for StructParser {
    type Output = Option<ModuleItem>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let name = state.consume_identifier("struct name")?;

        let generic_parameters = GenericParametersParser.optionally_parse(state)?;

        if state.next_token.raw == Punctuator::OpenParent {
            let fields = TupleFieldsParser {
                context: ModuleItemKind::Struct,
            }
            .parse(state)?;

            let implements = if state.next_token.raw == Keyword::Implements {
                state.advance();

                Some(BoundsParser.parse(state)?)
            } else {
                None
            };

            let where_predicates = WherePredicatesParser.optionally_parse(state)?;

            let mut methods = vec![];

            if state.next_token.raw != Punctuator::Semicolon {
                state.consume(Punctuator::OpenBrace, "struct")?;

                loop {
                    if state.next_token.raw == Punctuator::CloseBrace {
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
        } else if state.next_token.raw == Punctuator::OpenBrace {
            let implements = if state.next_token.raw == Keyword::Implements {
                state.advance();

                Some(BoundsParser.parse(state)?)
            } else {
                None
            };

            let where_predicates = WherePredicatesParser.optionally_parse(state)?;

            let fields = parse_list!(
                state,
                node_name: "struct fields",
                closing_token: one_of(Punctuator::CloseBrace, Keyword::Fun),
                parse_element_expr: {
                    let docstring = state.consume_local_docstring();
                    let visibility = VisibilityParser.parse(state);

                    StructFieldParser {
                        visibility,
                        docstring,
                    }
                    .parse(state)
                }
            );

            let mut methods = vec![];

            if state.next_token.raw == Keyword::Fun {
                loop {
                    if state.next_token.raw == Punctuator::CloseBrace {
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
                expected!(Punctuator::Semicolon, Punctuator::OpenParent),
                "item",
            ));

            None
        }
    }
}

struct NotSelfFunctionParameterParser;

impl Parse for NotSelfFunctionParameterParser {
    type Output = Option<NotSelfFunctionParameter>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let name = state.consume_identifier("function parameter name")?;

        state.consume(Punctuator::Colon, "function parameter name")?;

        let ty = TypeParser.parse(state)?;

        Some(NotSelfFunctionParameter { name, ty })
    }
}

struct FunctionParser {
    visibility: Visibility,
    docstring: Option<String>,
}

impl Parse for FunctionParser {
    type Output = Option<Function>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.consume(Keyword::Fun, "function")?;

        let name = state.consume_identifier("function name")?;

        let generic_parameters = GenericParametersParser.optionally_parse(state)?;

        state.consume(Punctuator::OpenParent, "function")?;

        let parameters = parse_list!(
            state,
            node_name: "function parameters",
            closing_token: Punctuator::CloseParent,
            parse_element_expr: {
                if state.lexer.scanned_identifier == builtin_identifiers::SMALL_SELF {
                    state.advance();

                    Some(FunctionParameter::SelfParameter(SelfFunctionParameter {
                        self_location: state.current_token.location,
                        ty: if state.next_token.raw == Punctuator::Colon {
                            state.advance();

                            Some(TypeParser.parse(state)?)
                        } else {
                            None
                        },
                    }))
                } else {
                    NotSelfFunctionParameterParser.parse(state)
                        .map(FunctionParameter::NotSelfParameter)
                }
            }
        );

        state.advance();

        let return_type = if state.next_token.raw == Punctuator::Colon {
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
                RawToken::Punctuator(Punctuator::Semicolon) => {
                    state.advance();

                    None
                }
                RawToken::Punctuator(Punctuator::OpenBrace) => StatementsBlockParser.parse(state),
                _ => {
                    state.advance();

                    state.add_diagnostic(UnexpectedTokenDiagnostic::new(
                        state.current_token,
                        expected!(Punctuator::Semicolon, Punctuator::OpenParent),
                        "function",
                    ));

                    return None;
                }
            },
        })
    }
}

struct TypeAliasParser {
    visibility: Visibility,
    docstring: Option<String>,
}

impl Parse for TypeAliasParser {
    type Output = Option<TypeAlias>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let name = state.consume_identifier("type alias")?;
        let generic_parameters = GenericParametersParser.optionally_parse(state)?;

        state.consume(Punctuator::Eq, "type alias")?;

        let value = TypeParser.parse(state)?;

        state.consume(Punctuator::Semicolon, "type alias")?;

        Some(TypeAlias {
            visibility: self.visibility,
            name,
            generic_parameters,
            value,
            docstring: self.docstring,
        })
    }
}

struct InterfaceParser {
    visibility: Visibility,
    docstring: Option<String>,
}

impl Parse for InterfaceParser {
    type Output = Option<ModuleItem>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let name = state.consume_identifier("interface name in interface declaration")?;

        let generic_parameters = GenericParametersParser.optionally_parse(state)?;

        let implements = if state.next_token.raw == Keyword::Implements {
            state.advance();

            Some(BoundsParser.parse(state)?)
        } else {
            None
        };

        let where_predicates = WherePredicatesParser.optionally_parse(state)?;

        state.consume(Punctuator::OpenBrace, "interface declaration")?;

        let methods = parse_list!(
            state,
            node_name: "interface methods",
            closing_token: Punctuator::CloseBrace,
            parse_element_expr: {
                let method = FunctionParser {
                    docstring: state.consume_local_docstring(),
                    visibility: VisibilityParser.parse(state),
                }
                .parse(state)?;

                if let Visibility::Public(location) = method.signature.visibility {
                    state.add_diagnostic(UnnecessaryVisibilityQualifierDiagnostic {
                        location,
                        context: UnnecessaryVisibilityQualifierContext::InterfaceMethod {
                            name_location: method.signature.name.location
                        },
                    });
                }

                Some(method)
            }
        );

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

struct EnumParser {
    visibility: Visibility,
    docstring: Option<String>,
}

impl Parse for EnumParser {
    type Output = Option<ModuleItem>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let name = state.consume_identifier("enum name")?;

        let generic_parameters = GenericParametersParser.optionally_parse(state)?;

        let implements = if state.next_token.raw == Keyword::Implements {
            state.advance();

            Some(BoundsParser.parse(state)?)
        } else {
            None
        };

        let where_predicates = WherePredicatesParser.optionally_parse(state)?;

        state.consume(Punctuator::OpenBrace, "enum")?;

        let items = parse_list!(
            state,
            node_name: "enum items",
            closing_token: one_of(Punctuator::CloseBrace, Keyword::Fun),
            parse_element_expr: EnumItemParser.parse(state)
        );

        let mut methods = vec![];

        loop {
            if state.next_token.raw == Punctuator::CloseBrace {
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

struct EnumItemParser;

impl Parse for EnumItemParser {
    type Output = Option<EnumItem>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let docstring = state.consume_local_docstring();

        let name = state.consume_identifier("enum item")?;

        match state.next_token.raw {
            RawToken::Punctuator(Punctuator::OpenBrace) => {
                EnumItemStructParser { name, docstring }.parse(state)
            }
            RawToken::Punctuator(Punctuator::OpenParent) => Some(EnumItem::TupleLike {
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

struct EnumItemStructParser {
    name: IdentifierAST,
    docstring: Option<String>,
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

struct TupleFieldsParser {
    context: ModuleItemKind,
}

impl Parse for TupleFieldsParser {
    type Output = Option<Vec<TupleField>>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let fields = parse_list!(
            state,
            node_name: format!("item tuple in {}", self.context.to_string()),
            closing_token: Punctuator::CloseParent,
            parse_element_expr:
                Some(TupleField {
                    visibility: VisibilityParser.parse(state),
                    ty: TypeParser.parse(state)?,
                })
        );

        state.advance(); // `)`

        Some(fields)
    }
}

pub(crate) struct ItemsParser;

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
                RawToken::Keyword(
                    Keyword::Enum
                    | Keyword::Import
                    | Keyword::Struct
                    | Keyword::Fun
                    | Keyword::Type
                    | Keyword::Interface,
                )
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

pub(crate) struct ItemParser;

impl Parse for ItemParser {
    type Output = Option<ModuleItem>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let docstring = state.consume_local_docstring();
        let visibility = VisibilityParser.parse(state);

        Some(match state.next_token.raw {
            RawToken::Keyword(Keyword::Enum) => {
                possibly_recover!(
                    state,
                    EnumParser {
                        visibility,
                        docstring
                    }
                    .parse(state)
                )
            }
            RawToken::Keyword(Keyword::Import) => {
                possibly_recover!(state, ImportParser { visibility }.parse(state))
            }
            RawToken::Keyword(Keyword::Struct) => {
                possibly_recover!(
                    state,
                    StructParser {
                        visibility,
                        docstring
                    }
                    .parse(state)
                )
            }
            RawToken::Keyword(Keyword::Interface) => {
                possibly_recover!(
                    state,
                    InterfaceParser {
                        visibility,
                        docstring
                    }
                    .parse(state)
                )
            }
            RawToken::Keyword(Keyword::Fun) => ModuleItem::Function(possibly_recover!(
                state,
                FunctionParser {
                    visibility,
                    docstring
                }
                .parse(state)
            )),
            RawToken::Keyword(Keyword::Type) => ModuleItem::TypeAlias(possibly_recover!(
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
                        Keyword::Enum,
                        Keyword::Import,
                        Keyword::Struct,
                        Keyword::Interface,
                        Keyword::Fun,
                        Keyword::Type
                    ),
                    "item",
                ));

                Self::goto_next_valid_item(state);
                return None;
            }
        })
    }
}
