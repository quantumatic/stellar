use stellar_ast::{
    token::{Keyword, Punctuator, RawToken},
    GenericParameter, Type, TypeConstructor, WherePredicate,
};

use crate::{
    diagnostics::UnexpectedToken, list::ListParser, path::PathParser, OptionallyParse, Parse,
    ParseState,
};

pub(crate) struct BoundsParser;

impl Parse for BoundsParser {
    type Output = Vec<TypeConstructor>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        let mut bounds = vec![];

        if let Some(b) = TypeConstructorParser.parse(state) {
            bounds.push(b);
        }

        while state.next_token.raw == Punctuator::Plus {
            state.advance();

            if let Some(bound) = TypeConstructorParser.parse(state) {
                bounds.push(bound);
            }
        }

        bounds
    }
}

pub(crate) struct TypeParser;

impl Parse for TypeParser {
    type Output = Option<Type>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        match state.next_token.raw {
            RawToken::Punctuator(Punctuator::OpenParent) => {
                ParenthesizedOrTupleTypeParser.parse(state)
            }
            RawToken::Identifier => TypeConstructorParser.parse(state).map(Type::Constructor),
            RawToken::Keyword(Keyword::Dyn) => InterfaceObjectTypeParser.parse(state),
            RawToken::Punctuator(Punctuator::Underscore) => {
                state.advance();

                Some(Type::Underscore {
                    location: state.current_token.location,
                })
            }
            RawToken::Keyword(Keyword::Fun) => FunctionTypeParser.parse(state),
            _ => {
                state.diagnostics.add_diagnostic(UnexpectedToken::new(
                    state.current_token.location.end,
                    state.next_token,
                    "type",
                ));

                None
            }
        }
    }
}

struct InterfaceObjectTypeParser;

impl Parse for InterfaceObjectTypeParser {
    type Output = Option<Type>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        let start = state.next_token.location.start;

        state.advance(); // `dyn`

        Some(Type::InterfaceObject {
            bounds: BoundsParser.parse(state),
            location: state.location_from(start),
        })
    }
}

struct FunctionTypeParser;

impl Parse for FunctionTypeParser {
    type Output = Option<Type>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        let start = state.next_token.location.start;

        state.advance(); // `fun`

        state.consume(Punctuator::OpenParent)?;

        let parameter_types =
            ListParser::new(&[RawToken::from(Punctuator::CloseParent)], |state| {
                TypeParser.parse(state)
            })
            .parse(state)?;

        state.advance(); // `)`

        let return_type = if state.next_token.raw == Punctuator::Colon {
            state.advance();

            Some(Box::new(TypeParser.parse(state)?))
        } else {
            None
        };

        Some(Type::Function {
            location: state.location_from(start),
            parameter_types,
            return_type,
        })
    }
}

struct ParenthesizedOrTupleTypeParser;

impl Parse for ParenthesizedOrTupleTypeParser {
    type Output = Option<Type>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        let start = state.next_token.location.start;
        state.advance(); // `(`

        let element_types = ListParser::new(&[RawToken::from(Punctuator::CloseParent)], |state| {
            TypeParser.parse(state)
        })
        .parse(state)?;

        state.advance(); // `)`

        let location = state.location_from(start);

        let mut element_types = element_types.into_iter();

        match (element_types.next(), element_types.next()) {
            (Some(element), None) => {
                if state
                    .resolve_location(state.location_from(element.location().end))
                    .contains(',')
                {
                    Some(Type::Tuple {
                        location,
                        element_types: vec![element],
                    })
                } else {
                    Some(Type::Parenthesized {
                        location,
                        inner: Box::from(element),
                    })
                }
            }
            (None, None) => Some(Type::Tuple {
                location,
                element_types: vec![],
            }),
            (Some(previous), Some(next)) => {
                let mut new_element_types = vec![];
                new_element_types.push(previous);
                new_element_types.push(next);

                new_element_types.append(&mut element_types.collect::<Vec<_>>());

                Some(Type::Tuple {
                    location,
                    element_types: new_element_types,
                })
            }
            _ => unreachable!(),
        }
    }
}

pub(crate) struct GenericParametersParser;

impl OptionallyParse for GenericParametersParser {
    type Output = Option<Vec<GenericParameter>>;

    fn optionally_parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        if state.next_token.raw != Punctuator::OpenBracket {
            return Some(vec![]);
        }

        state.advance();

        let result = ListParser::new(&[RawToken::from(Punctuator::CloseBracket)], |state| {
            Some(GenericParameter {
                name: state.consume_identifier()?,
                bounds: if state.next_token.raw == Punctuator::Colon {
                    state.advance();

                    Some(BoundsParser.parse(state))
                } else {
                    None
                },
                default_value: if state.next_token.raw == Punctuator::Eq {
                    state.advance();

                    Some(TypeParser.parse(state)?)
                } else {
                    None
                },
            })
        })
        .parse(state)?;

        state.advance();

        Some(result)
    }
}

pub(crate) struct TypeConstructorParser;

impl Parse for TypeConstructorParser {
    type Output = Option<TypeConstructor>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        let path = PathParser.parse(state)?;
        let arguments = TypeArgumentsParser.optionally_parse(state)?;

        Some(TypeConstructor {
            location: state.location_from(path.location.start),
            path,
            arguments,
        })
    }
}

pub(crate) struct TypeArgumentsParser;

impl OptionallyParse for TypeArgumentsParser {
    type Output = Option<Vec<Type>>;

    fn optionally_parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        if state.next_token.raw != Punctuator::OpenBracket {
            return Some(vec![]);
        }

        self.parse(state)
    }
}

impl Parse for TypeArgumentsParser {
    type Output = Option<Vec<Type>>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        state.advance();

        let result = ListParser::new(&[RawToken::from(Punctuator::CloseBracket)], |state| {
            TypeParser.parse(state)
        })
        .parse(state)?;

        state.advance();

        Some(result)
    }
}

pub(crate) struct WherePredicatesParser;

impl OptionallyParse for WherePredicatesParser {
    type Output = Option<Vec<WherePredicate>>;

    fn optionally_parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        if state.next_token.raw != Keyword::Where {
            return Some(vec![]);
        }

        state.advance();

        ListParser::new(
            &[
                RawToken::from(Punctuator::OpenBrace),
                RawToken::from(Punctuator::Semicolon),
            ],
            |state| {
                let left = TypeParser.parse(state)?;

                state.consume(Punctuator::Colon)?;

                Some(WherePredicate {
                    ty: left,
                    bounds: BoundsParser.parse(state),
                })
            },
        )
        .parse(state)
    }
}
