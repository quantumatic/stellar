use stellar_ast::{
    token::{Punctuator, RawToken},
    NegativeNumericLiteral, Path, Pattern, StructFieldPattern,
};

use crate::{
    diagnostics::{FloatOverflow, IntegerOverflow, UnexpectedToken},
    list::ListParser,
    literal::LiteralParser,
    path::PathParser,
    Parse, ParseState,
};

pub(crate) struct PatternParser;

impl Parse for PatternParser {
    type Output = Option<Pattern>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        let left = PatternExceptOrParser.parse(state)?;

        if state.next_token.raw == Punctuator::Or {
            state.advance();

            let right = Self.parse(state)?;

            Some(Pattern::Or {
                location: state.make_location(left.location().start, right.location().end),
                left: Box::new(left),
                right: Box::new(right),
            })
        } else {
            Some(left)
        }
    }
}

struct PatternExceptOrParser;

impl Parse for PatternExceptOrParser {
    type Output = Option<Pattern>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        match state.next_token.raw {
            RawToken::StringLiteral
            | RawToken::CharLiteral
            | RawToken::IntegerLiteral
            | RawToken::FloatLiteral
            | RawToken::TrueBoolLiteral
            | RawToken::FalseBoolLiteral => LiteralParser.parse(state).map(Pattern::Literal),
            RawToken::Punctuator(Punctuator::Minus) => {
                state.advance();

                match state.next_token.raw {
                    RawToken::IntegerLiteral => {
                        state.advance();

                        if let Ok(value) = state.resolve_current().replace('_', "").parse::<u64>() {
                            Some(NegativeNumericLiteral::Integer {
                                value,
                                location: state.current_token.location,
                            })
                        } else {
                            state
                                .add_diagnostic(IntegerOverflow::new(state.current_token.location));

                            None
                        }
                    }
                    RawToken::FloatLiteral => {
                        state.advance();

                        if let Ok(value) = state.resolve_current().replace('_', "").parse::<f64>() {
                            Some(NegativeNumericLiteral::Float {
                                value,
                                location: state.current_token.location,
                            })
                        } else {
                            state.add_diagnostic(FloatOverflow::new(state.current_token.location));

                            None
                        }
                    }
                    _ => {
                        state.add_diagnostic(UnexpectedToken::new(
                            state.current_token.location.end,
                            state.next_token,
                            "numeric literal",
                        ));

                        None
                    }
                }
                .map(Pattern::NegativeNumericLiteral)
            }
            RawToken::Punctuator(Punctuator::Underscore) => {
                state.advance();

                Some(Pattern::Wildcard {
                    location: state.current_token.location,
                })
            }
            RawToken::Identifier => {
                let path = PathParser.parse(state)?;

                match state.next_token.raw {
                    RawToken::Punctuator(Punctuator::OpenBrace) => {
                        return StructPatternParser { path }.parse(state);
                    }
                    RawToken::Punctuator(Punctuator::OpenParent) => {
                        return TupleLikePatternParser { path }.parse(state);
                    }
                    _ => {}
                };

                // If it is only 1 identifier
                if path.identifiers.len() == 1 {
                    let identifier = path.identifiers.first().expect(
                        "Cannot get first identifier in path when parsing identifier pattern",
                    );

                    let pattern = if state.next_token.raw == Punctuator::At {
                        state.advance();
                        Some(Box::new(PatternParser.parse(state)?))
                    } else {
                        None
                    };

                    Some(Pattern::Identifier {
                        location: state.make_location(
                            path.location.start,
                            match pattern {
                                Some(ref pattern) => pattern.location().end,
                                None => path.location.end,
                            },
                        ),
                        identifier: *identifier,
                        pattern,
                    })
                } else {
                    Some(Pattern::Path { path })
                }
            }
            RawToken::Punctuator(Punctuator::OpenBracket) => ListPatternParser.parse(state),
            RawToken::Punctuator(Punctuator::DoubleDot) => {
                state.advance();

                Some(Pattern::Rest {
                    location: state.next_token.location,
                })
            }
            RawToken::Punctuator(Punctuator::OpenParent) => {
                GroupedOrTuplePatternParser.parse(state)
            }
            _ => {
                state.add_diagnostic(UnexpectedToken::new(
                    state.current_token.location.end,
                    state.next_token,
                    "pattern",
                ));
                None
            }
        }
    }
}

struct StructPatternParser {
    pub(crate) path: Path,
}

impl Parse for StructPatternParser {
    type Output = Option<Pattern>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        state.advance(); // `{`

        let fields = ListParser::new(&[RawToken::from(Punctuator::CloseBrace)], |state| {
            if state.next_token.raw == Punctuator::DoubleDot {
                state.advance();

                Some(StructFieldPattern::Rest {
                    location: state.current_token.location,
                })
            } else {
                let field_name = state.consume_identifier()?;

                let value_pattern = if state.next_token.raw == Punctuator::Colon {
                    state.advance();

                    Some(PatternParser.parse(state)?)
                } else {
                    None
                };

                Some(StructFieldPattern::NotRest {
                    location: state.location_from(field_name.location.start),
                    field_name,
                    value_pattern,
                })
            }
        })
        .parse(state)?;

        state.advance();

        Some(Pattern::Struct {
            location: state.location_from(self.path.location.start),
            path: self.path,
            fields,
        })
    }
}

struct ListPatternParser;

impl Parse for ListPatternParser {
    type Output = Option<Pattern>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        let start = state.next_token.location.start;
        state.advance();

        let inner_patterns =
            ListParser::new(&[RawToken::from(Punctuator::CloseBracket)], |state| {
                PatternParser.parse(state)
            })
            .parse(state)?;

        state.advance();

        Some(Pattern::List {
            location: state.location_from(start),
            inner_patterns,
        })
    }
}

struct TupleLikePatternParser {
    pub(crate) path: Path,
}

impl Parse for TupleLikePatternParser {
    type Output = Option<Pattern>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        state.advance(); // `(`

        let inner_patterns = ListParser::new(&[RawToken::from(Punctuator::CloseParent)], |state| {
            PatternParser.parse(state)
        })
        .parse(state)?;

        state.advance(); // `)`

        Some(Pattern::TupleLike {
            location: state.location_from(self.path.location.start),
            path: self.path,
            inner_patterns,
        })
    }
}

struct GroupedOrTuplePatternParser;

impl Parse for GroupedOrTuplePatternParser {
    type Output = Option<Pattern>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        let start = state.next_token.location.start;
        state.advance();

        let elements = ListParser::new(&[RawToken::from(Punctuator::CloseParent)], |state| {
            PatternParser.parse(state)
        })
        .parse(state)?;

        state.advance();

        let location = state.location_from(start);

        let mut elements = elements.into_iter();

        match (elements.next(), elements.next()) {
            (Some(element), None) => {
                if element.is_rest()
                    | state
                        .resolve_location(state.location_from(element.location().end))
                        .contains(',')
                {
                    Some(Pattern::Tuple {
                        location,
                        elements: vec![element],
                    })
                } else {
                    Some(Pattern::Grouped {
                        location,
                        inner: Box::from(element),
                    })
                }
            }
            (None, None) => Some(Pattern::Tuple {
                location,
                elements: vec![],
            }),
            (Some(previous), Some(next)) => {
                let mut new_elements = vec![];
                new_elements.push(previous);
                new_elements.push(next);

                new_elements.append(&mut elements.collect::<Vec<_>>());

                Some(Pattern::Tuple {
                    location,
                    elements: new_elements,
                })
            }
            _ => unreachable!(),
        }
    }
}