use ry_ast::{
    token::RawToken, EnumItem, Function, FunctionParameter, FunctionParameterType,
    FunctionSignature, IdentifierAST, Impl, ModuleItem, ModuleItemKind, NotSelfFunctionParameter,
    SelfFunctionParameter, StructField, Token, TraitItem, TupleField, TypeAlias, Visibility,
};
use ry_filesystem::location::Location;
use ry_interner::builtin_symbols;

use crate::{
    diagnostics::{
        UnexpectedTokenDiagnostic, UnnecessaryVisibilityQualifierContext,
        UnnecessaryVisibilityQualifierDiagnostic,
    },
    expected,
    macros::parse_list,
    path::ImportPathParser,
    r#type::{TypeParametersParser, TypeBoundsParser, TypeParser, WherePredicatesParser},
    statement::StatementsBlockParser,
    OptionalParser, Parse, ParseState, VisibilityParser,
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
    pub(crate) docstring: Option<String>,
}

struct FunctionParser {
    pub(crate) visibility: Visibility,
    pub(crate) docstring: Option<String>,
}

pub(crate) struct FunctionParameterTypeParser;

pub(crate) struct NotSelfFunctionParameterParser;

struct TypeAliasParser {
    pub(crate) visibility: Visibility,
    pub(crate) docstring: Option<String>,
}

struct TraitParser {
    pub(crate) visibility: Visibility,
    pub(crate) docstring: Option<String>,
}

struct TraitItemsParser {
    pub(crate) name_location: Location,
    pub(crate) type_implementation: bool,
}

struct ImplParser {
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

        if let Some(location) = self.visibility.location_of_pub() {
            state.save_single_file_diagnostic(UnnecessaryVisibilityQualifierDiagnostic {
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
        let visibility = VisibilityParser.parse(state);

        let name = state.consume_identifier("struct field")?;

        state.consume(Token![:], "struct field")?;

        let ty = TypeParser.parse(state)?;

        Some(StructField {
            visibility,
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

        let type_parameters = TypeParametersParser.optionally_parse(state)?;

        let where_predicates = WherePredicatesParser.optionally_parse(state)?;

        if state.next_token.raw == Token!['{'] {
            let fields = StructFieldsParser.parse(state)?;
            Some(ModuleItem::Struct {
                visibility: self.visibility,
                name,
                type_parameters,
                where_predicates,
                fields,
                docstring: self.docstring,
            })
        } else if state.next_token.raw == Token!['('] {
            let fields = TupleFieldsParser {
                context: ModuleItemKind::Struct,
            }
            .parse(state)?;

            if state.next_token.raw == Token![;] {
                state.advance();
            } else {
                state.save_single_file_diagnostic(UnexpectedTokenDiagnostic::new(
                    state.current_token,
                    expected!(Token![;]),
                    "struct item",
                ));
            }

            Some(ModuleItem::TupleLikeStruct {
                visibility: self.visibility,
                name,
                type_parameters,
                where_predicates,
                fields,
                docstring: self.docstring,
            })
        } else {
            state.save_single_file_diagnostic(UnexpectedTokenDiagnostic::new(
                state.current_token,
                expected!(Token![;], Token!['(']),
                "item",
            ));

            None
        }
    }
}

impl Parse for FunctionParameterTypeParser {
    type Output = Option<FunctionParameterType>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        if state.next_token.raw == Token![impl] {
            state.advance();

            let bounds = TypeBoundsParser.parse(state)?;

            Some(FunctionParameterType::Impl(bounds))
        } else {
            TypeParser.parse(state).map(FunctionParameterType::Type)
        }
    }
}

impl Parse for NotSelfFunctionParameterParser {
    type Output = Option<NotSelfFunctionParameter>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let name = state.consume_identifier("function parameter name")?;

        state.consume(Token![:], "function parameter name")?;

        let ty = FunctionParameterTypeParser.parse(state)?;

        Some(NotSelfFunctionParameter { name, ty })
    }
}

impl Parse for FunctionParser {
    type Output = Option<Function>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let name = state.consume_identifier("function name")?;

        let type_parameters = TypeParametersParser.optionally_parse(state)?;

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
                type_parameters,
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

                    state.save_single_file_diagnostic(UnexpectedTokenDiagnostic::new(
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

impl Parse for TraitItemsParser {
    type Output = Option<(Vec<TraitItem>, bool)>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let mut items = vec![];

        while state.next_token.raw != Token!['}'] {
            let docstring = state.consume_local_docstring();

            if let Some(location) = VisibilityParser.parse(state).location_of_pub() {
                if !self.type_implementation {
                    state.save_single_file_diagnostic(UnnecessaryVisibilityQualifierDiagnostic {
                        location,
                        context: UnnecessaryVisibilityQualifierContext::TraitItem {
                            name_location: self.name_location,
                        },
                    });
                }
            }

            items.push(match state.next_token.raw {
                Token![fun] => Some(TraitItem::AssociatedFunction(
                    FunctionParser {
                        visibility: Visibility::private(),
                        docstring,
                    }
                    .parse(state)?,
                )),
                Token![type] => Some(TraitItem::TypeAlias(
                    TypeAliasParser {
                        visibility: Visibility::private(),
                        docstring,
                    }
                    .parse(state)?,
                )),
                _ => {
                    state.save_single_file_diagnostic(UnexpectedTokenDiagnostic::new(
                        state.next_token,
                        expected!(Token![fun], Token![type]),
                        "trait item",
                    ));
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
        let type_parameters = TypeParametersParser.optionally_parse(state)?;

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
            type_parameters,
            bounds,
            value,
            docstring: self.docstring,
        })
    }
}

impl Parse for TraitParser {
    type Output = Option<ModuleItem>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let name = state.consume_identifier("trait name in trait declaration")?;

        let type_parameters = TypeParametersParser.optionally_parse(state)?;

        let where_predicates = WherePredicatesParser.optionally_parse(state)?;

        state.consume(Token!['{'], "trait declaration")?;

        let items = TraitItemsParser {
            name_location: name.location,
            type_implementation: false,
        }
        .parse(state)?;

        if !items.1 {
            state.consume(Token!['}'], "trait declaration")?;
        }

        Some(ModuleItem::Trait {
            visibility: self.visibility,
            name,
            type_parameters,
            where_predicates,
            items: items.0,
            docstring: self.docstring,
        })
    }
}

impl Parse for ImplParser {
    type Output = Option<ModuleItem>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let location = state.current_token.location;

        if let Some(location) = self.visibility.location_of_pub() {
            state.save_single_file_diagnostic(UnnecessaryVisibilityQualifierDiagnostic {
                location,
                context: UnnecessaryVisibilityQualifierContext::Impl,
            });
        }

        let type_parameters = TypeParametersParser.optionally_parse(state)?;

        let mut ty = TypeParser.parse(state)?;
        let mut r#trait = None;

        if state.next_token.raw == Token![for] {
            state.advance();

            r#trait = Some(ty);
            ty = TypeParser.parse(state)?;
        }

        let where_predicates = WherePredicatesParser.optionally_parse(state)?;

        state.consume(Token!['{'], "type implementation")?;

        let items = TraitItemsParser {
            name_location: location,
            type_implementation: true,
        }
        .parse(state)?;

        if !items.1 {
            state.consume(Token!['}'], "type implementation")?;
        }

        Some(ModuleItem::Impl(Impl {
            location,
            type_parameters,
            ty,
            r#trait,
            where_predicates,
            items: items.0,
            docstring: self.docstring,
        }))
    }
}

impl Parse for EnumParser {
    type Output = Option<ModuleItem>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let name = state.consume_identifier("enum name")?;

        let type_parameters = TypeParametersParser.optionally_parse(state)?;

        state.consume(Token!['{'], "enum")?;

        let items = parse_list!(state, "enum items", Token!['}'], {
            Some(EnumItemParser.parse(state)?)
        });

        state.advance(); // `}`

        let where_predicates = WherePredicatesParser.optionally_parse(state)?;

        Some(ModuleItem::Enum {
            visibility: self.visibility,
            name,
            type_parameters,
            where_predicates,
            items,
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
    fn go_to_next_item(state: &mut ParseState<'_, '_, '_>) {
        loop {
            match state.next_token.raw {
                Token![enum]
                | Token![import]
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
    type Output = Option<ModuleItem>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let docstring = state.consume_local_docstring();
        let visibility = VisibilityParser.parse(state);

        Some(match state.next_token.raw {
            Token![enum] => {
                go_to_next_valid_item!(
                    state,
                    EnumParser {
                        visibility,
                        docstring
                    }
                    .parse(state)
                )
            }
            Token![import] => {
                go_to_next_valid_item!(state, ImportParser { visibility }.parse(state))
            }
            Token![struct] => {
                go_to_next_valid_item!(
                    state,
                    StructParser {
                        visibility,
                        docstring
                    }
                    .parse(state)
                )
            }
            Token![trait] => {
                go_to_next_valid_item!(
                    state,
                    TraitParser {
                        visibility,
                        docstring
                    }
                    .parse(state)
                )
            }
            Token![fun] => ModuleItem::Function(go_to_next_valid_item!(
                state,
                FunctionParser {
                    visibility,
                    docstring
                }
                .parse(state)
            )),
            Token![impl] => {
                go_to_next_valid_item!(
                    state,
                    ImplParser {
                        visibility,
                        docstring
                    }
                    .parse(state)
                )
            }
            Token![type] => ModuleItem::TypeAlias(go_to_next_valid_item!(
                state,
                TypeAliasParser {
                    visibility,
                    docstring
                }
                .parse(state)
            )),
            _ => {
                state.save_single_file_diagnostic(UnexpectedTokenDiagnostic::new(
                    state.next_token,
                    expected!(
                        Token![import],
                        Token![fun],
                        Token![trait],
                        Token![enum],
                        Token![struct],
                        Token![impl],
                        Token![type],
                        RawToken::EndOfFile
                    ),
                    "item",
                ));

                loop {
                    match state.next_token.raw {
                        Token![enum]
                        | Token![import]
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
