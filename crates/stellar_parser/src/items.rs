use stellar_ast::{
    token::{Keyword, Punctuator, RawToken},
    Enum, EnumItem, Function, FunctionParameter, FunctionSignature, IdentifierAST, Interface,
    ModuleItem, NotSelfFunctionParameter, SelfFunctionParameter, Struct, StructField, TupleField,
    TupleLikeStruct, TypeAlias, Visibility,
};
use stellar_english_commons::enumeration::one_of;
use stellar_interner::builtin_identifiers;

use crate::{
    diagnostics::{
        UnnecessaryVisibilityQualifierContext, UnnecessaryVisibilityQualifierDiagnostic,
    },
    list::ListParser,
    path::ImportPathParser,
    pattern::PatternParser,
    r#type::{
        BoundsParser, GenericParametersParser, TypeConstructorParser, TypeParser,
        WherePredicatesParser,
    },
    statement::StatementsBlockParser,
    OptionallyParse, Parse, ParseState, VisibilityParser,
};

struct ImportParser {
    visibility: Visibility,
}

impl Parse for ImportParser {
    type Output = Option<ModuleItem>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        let start = state.next_token.location.start;

        if let Visibility::Public(location) = self.visibility {
            state
                .diagnostics
                .add_diagnostic(UnnecessaryVisibilityQualifierDiagnostic {
                    location,
                    context: UnnecessaryVisibilityQualifierContext::Import,
                });
        }

        state.advance();

        let path = ImportPathParser.parse(state)?;

        state.consume(Punctuator::Semicolon)?;

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

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        let name = state.consume_identifier()?;

        state.consume(Punctuator::Colon)?;

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

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        state.consume(Punctuator::OpenBrace)?;

        let fields = ListParser::new(&[RawToken::from(Punctuator::CloseBrace)], |state| {
            StructFieldParser {
                docstring: state.consume_local_docstring(),
                visibility: VisibilityParser.parse(state),
            }
            .parse(state)
        })
        .parse(state)?;

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

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        state.advance();

        let name = state.consume_identifier()?;

        let generic_parameters = GenericParametersParser.optionally_parse(state)?;

        let implements = if state.next_token.raw == Keyword::Implements {
            state.advance();

            Some(
                ListParser::new(
                    &[
                        RawToken::from(Keyword::Where),
                        RawToken::from(Punctuator::Semicolon),
                        RawToken::from(Punctuator::OpenBrace),
                    ],
                    |state| TypeConstructorParser.parse(state),
                )
                .parse(state)?,
            )
        } else {
            None
        };

        let where_predicates = WherePredicatesParser.optionally_parse(state)?;

        if state.next_token.raw == Punctuator::OpenParent
            && where_predicates.is_empty()
            && implements.is_none()
        {
            let fields = TupleFieldsParser.parse(state)?;

            let implements = if state.next_token.raw == Keyword::Implements {
                state.advance();

                Some(
                    ListParser::new(
                        &[
                            RawToken::from(Keyword::Where),
                            RawToken::from(Punctuator::Semicolon),
                            RawToken::from(Punctuator::OpenBrace),
                        ],
                        |state| TypeConstructorParser.parse(state),
                    )
                    .parse(state)?,
                )
            } else {
                None
            };

            let where_predicates = WherePredicatesParser.optionally_parse(state)?;

            let mut methods = vec![];

            if state.next_token.raw != Punctuator::Semicolon {
                state.consume(Punctuator::OpenBrace)?;

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

            Some(ModuleItem::TupleLikeStruct(TupleLikeStruct {
                visibility: self.visibility,
                name,
                generic_parameters,
                where_predicates,
                fields,
                methods,
                implements,
                docstring: self.docstring,
            }))
        } else if state.next_token.raw == Punctuator::OpenBrace {
            state.advance();

            let fields = ListParser::new(
                &[
                    RawToken::from(Punctuator::CloseBrace),
                    RawToken::from(Keyword::Fun),
                    RawToken::from(Keyword::Pub),
                ],
                |state| {
                    let docstring = state.consume_local_docstring();
                    let visibility = VisibilityParser.parse(state);

                    StructFieldParser {
                        visibility,
                        docstring,
                    }
                    .parse(state)
                },
            )
            .parse(state)?;

            let mut methods = vec![];

            if state.next_token.raw == Keyword::Fun || state.next_token.raw == Keyword::Pub {
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

            state.advance();

            Some(ModuleItem::Struct(Struct {
                visibility: self.visibility,
                name,
                generic_parameters,
                where_predicates,
                fields,
                methods,
                implements,
                docstring: self.docstring,
            }))
        } else {
            state.add_unexpected_token_diagnostic(one_of([
                Punctuator::Semicolon,
                Punctuator::OpenParent,
                Punctuator::OpenBrace,
            ]));

            None
        }
    }
}

struct NotSelfFunctionParameterParser;

impl Parse for NotSelfFunctionParameterParser {
    type Output = Option<NotSelfFunctionParameter>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        let pattern = PatternParser.parse(state)?;

        state.consume(Punctuator::Colon)?;

        let ty = TypeParser.parse(state)?;

        Some(NotSelfFunctionParameter { pattern, ty })
    }
}

struct FunctionParser {
    visibility: Visibility,
    docstring: Option<String>,
}

impl Parse for FunctionParser {
    type Output = Option<Function>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        state.consume(Keyword::Fun)?;

        let name = state.consume_identifier()?;

        let generic_parameters = GenericParametersParser.optionally_parse(state)?;

        state.consume(Punctuator::OpenParent)?;

        let parameters = ListParser::new(&[RawToken::from(Punctuator::CloseParent)], |state| {
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
                NotSelfFunctionParameterParser
                    .parse(state)
                    .map(FunctionParameter::NotSelfParameter)
            }
        })
        .parse(state)?;

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
                RawToken::Punctuator(Punctuator::OpenBrace) => {
                    Some(StatementsBlockParser.parse(state)?)
                }
                _ => {
                    state.add_unexpected_token_diagnostic(one_of([
                        Punctuator::Semicolon,
                        Punctuator::OpenBrace,
                    ]));

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
    type Output = Option<ModuleItem>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        state.advance();

        let name = state.consume_identifier()?;
        let generic_parameters = GenericParametersParser.optionally_parse(state)?;

        state.consume(Punctuator::Eq)?;

        let value = TypeParser.parse(state)?;

        state.consume(Punctuator::Semicolon)?;

        Some(ModuleItem::TypeAlias(TypeAlias {
            visibility: self.visibility,
            name,
            generic_parameters,
            value,
            docstring: self.docstring,
        }))
    }
}

struct InterfaceParser {
    visibility: Visibility,
    docstring: Option<String>,
}

impl Parse for InterfaceParser {
    type Output = Option<ModuleItem>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        state.advance();

        let name = state.consume_identifier()?;

        let generic_parameters = GenericParametersParser.optionally_parse(state)?;

        let inherits = if state.next_token.raw == Punctuator::Colon {
            state.advance();

            Some(BoundsParser.parse(state))
        } else {
            None
        };

        let where_predicates = WherePredicatesParser.optionally_parse(state)?;

        state.consume(Punctuator::OpenBrace)?;

        let mut methods = vec![];

        loop {
            if state.next_token.raw == Punctuator::CloseBrace {
                break;
            }

            let method = FunctionParser {
                docstring: state.consume_local_docstring(),
                visibility: VisibilityParser.parse(state),
            }
            .parse(state)?;

            if let Visibility::Public(location) = method.signature.visibility {
                state
                    .diagnostics
                    .add_diagnostic(UnnecessaryVisibilityQualifierDiagnostic {
                        location,
                        context: UnnecessaryVisibilityQualifierContext::InterfaceMethod {
                            name_location: method.signature.name.location,
                        },
                    });
            }

            methods.push(method);
        }

        state.advance();

        Some(ModuleItem::Interface(Interface {
            visibility: self.visibility,
            name,
            generic_parameters,
            where_predicates,
            methods,
            inherits,
            docstring: self.docstring,
        }))
    }
}

struct EnumParser {
    visibility: Visibility,
    docstring: Option<String>,
}

macro_rules! possibly_recover {
    ($state:ident, $item:expr) => {
        if let Some(item) = $item {
            item
        } else {
            loop {
                match $state.next_token.raw {
                    RawToken::Keyword(
                        Keyword::Enum
                        | Keyword::Import
                        | Keyword::Struct
                        | Keyword::Type
                        | Keyword::Interface,
                    )
                    | RawToken::EndOfFile => break,
                    _ => $state.advance(),
                }
            }
            return None;
        }
    };
}

impl Parse for EnumParser {
    type Output = Option<ModuleItem>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        state.advance();

        let name = state.consume_identifier()?;

        let generic_parameters = GenericParametersParser.optionally_parse(state)?;

        let implements = if state.next_token.raw == Keyword::Implements {
            state.advance();

            Some(
                ListParser::new(
                    &[
                        RawToken::from(Keyword::Where),
                        RawToken::from(Punctuator::OpenBrace),
                    ],
                    |state| TypeConstructorParser.parse(state),
                )
                .parse(state)?,
            )
        } else {
            None
        };

        let where_predicates = WherePredicatesParser.optionally_parse(state)?;

        state.consume(Punctuator::OpenBrace)?;

        let items = ListParser::new(
            &[
                RawToken::from(Punctuator::CloseBrace),
                RawToken::from(Keyword::Fun),
                RawToken::from(Keyword::Pub),
            ],
            |state| EnumItemParser.parse(state),
        )
        .parse(state)?;

        let mut methods = vec![];

        loop {
            if state.next_token.raw == Punctuator::CloseBrace {
                break;
            }

            let docstring = state.consume_local_docstring();
            let visibility = VisibilityParser.parse(state);

            methods.push(possibly_recover!(
                state,
                FunctionParser {
                    visibility,
                    docstring,
                }
                .parse(state)
            ));
        }

        state.advance(); // `}`

        Some(ModuleItem::Enum(Enum {
            visibility: self.visibility,
            name,
            generic_parameters,
            where_predicates,
            items,
            methods,
            implements,
            docstring: self.docstring,
        }))
    }
}

struct EnumItemParser;

impl Parse for EnumItemParser {
    type Output = Option<EnumItem>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        let docstring = state.consume_local_docstring();

        let name = state.consume_identifier()?;

        match state.next_token.raw {
            RawToken::Punctuator(Punctuator::OpenBrace) => {
                EnumItemStructParser { name, docstring }.parse(state)
            }
            RawToken::Punctuator(Punctuator::OpenParent) => Some(EnumItem::TupleLike {
                name,
                fields: TupleFieldsParser.parse(state)?,
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

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        let fields = StructFieldsParser.parse(state)?;

        Some(EnumItem::Struct {
            name: self.name,
            fields,
            docstring: self.docstring,
        })
    }
}

struct TupleFieldsParser;

impl Parse for TupleFieldsParser {
    type Output = Option<Vec<TupleField>>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        state.advance(); // `(`

        let fields = ListParser::new(&[RawToken::from(Punctuator::CloseParent)], |state| {
            Some(TupleField {
                visibility: VisibilityParser.parse(state),
                ty: TypeParser.parse(state)?,
            })
        })
        .parse(state)?;

        state.advance(); // `)`

        Some(fields)
    }
}

pub(crate) struct ItemsParser;

impl Parse for ItemsParser {
    type Output = Vec<ModuleItem>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
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
    fn goto_next_valid_item(state: &mut ParseState<'_, '_>) {
        loop {
            match state.next_token.raw {
                RawToken::Keyword(
                    Keyword::Enum
                    | Keyword::Import
                    | Keyword::Struct
                    | Keyword::Type
                    | Keyword::Interface,
                )
                | RawToken::EndOfFile => break,
                _ => state.advance(),
            }
        }
    }
}

pub(crate) struct ItemParser;

impl Parse for ItemParser {
    type Output = Option<ModuleItem>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
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
            RawToken::Keyword(Keyword::Type) => possibly_recover!(
                state,
                TypeAliasParser {
                    visibility,
                    docstring
                }
                .parse(state)
            ),
            _ => {
                state.add_unexpected_token_diagnostic("module item");

                Self::goto_next_valid_item(state);

                return None;
            }
        })
    }
}
