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

impl PatternExceptOrParser {
    fn parse_negative_numeric_literal_pattern(
        &self,
        state: &mut ParseState<'_, '_>,
    ) -> Option<Pattern> {
        state.advance();

        match state.next_token.raw {
            RawToken::IntegerLiteral => {
                state.advance();

                if let Ok(value) = state
                    .resolve_current_token_str()
                    .replace('_', "")
                    .parse::<u64>()
                {
                    Some(NegativeNumericLiteral::Integer {
                        value,
                        location: state.current_token.location,
                    })
                } else {
                    state
                        .diagnostics
                        .add_diagnostic(IntegerOverflow::new(state.current_token.location));

                    None
                }
            }
            RawToken::FloatLiteral => {
                state.advance();

                if let Ok(value) = state
                    .resolve_current_token_str()
                    .replace('_', "")
                    .parse::<f64>()
                {
                    Some(NegativeNumericLiteral::Float {
                        value,
                        location: state.current_token.location,
                    })
                } else {
                    state
                        .diagnostics
                        .add_diagnostic(FloatOverflow::new(state.current_token.location));

                    None
                }
            }
            _ => {
                state.add_unexpected_token_diagnostic("numeric literal");

                None
            }
        }
        .map(Pattern::NegativeNumericLiteral)
    }

    fn parse_pattern_beginning_with_identifier(
        &self,
        state: &mut ParseState<'_, '_>,
    ) -> Option<Pattern> {
        let path = PathParser.parse(state)?;

        match state.next_token.raw {
            RawToken::Punctuator(Punctuator::OpenBrace) => {
                return self.parse_struct_pattern(state, path);
            }
            RawToken::Punctuator(Punctuator::OpenParent) => {
                return self.parse_tuple_like_struct_pattern(state, path);
            }
            _ => {}
        };

        // If it is only 1 identifier
        if path.identifiers.len() == 1 {
            let identifier = path
                .identifiers
                .first()
                .expect("Cannot get first identifier in path when parsing identifier pattern");

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

    fn parse_list_pattern(&self, state: &mut ParseState<'_, '_>) -> Option<Pattern> {
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

    fn parse_tuple_like_struct_pattern(
        &self,
        state: &mut ParseState<'_, '_>,
        path: Path,
    ) -> Option<Pattern> {
        state.advance(); // `(`

        let inner_patterns = ListParser::new(&[RawToken::from(Punctuator::CloseParent)], |state| {
            PatternParser.parse(state)
        })
        .parse(state)?;

        state.advance(); // `)`

        Some(Pattern::TupleLike {
            location: state.location_from(path.location.start),
            path,
            inner_patterns,
        })
    }

    fn parse_grouped_or_tuple_pattern(self, state: &mut ParseState<'_, '_>) -> Option<Pattern> {
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

    fn parse_struct_pattern(&self, state: &mut ParseState<'_, '_>, path: Path) -> Option<Pattern> {
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
            location: state.location_from(path.location.start),
            path,
            fields,
        })
    }
}

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
            RawToken::Punctuator(Punctuator::DoubleDot) => {
                state.advance();

                Some(Pattern::Rest {
                    location: state.next_token.location,
                })
            }
            RawToken::Punctuator(Punctuator::Underscore) => {
                state.advance();

                Some(Pattern::Wildcard {
                    location: state.current_token.location,
                })
            }
            RawToken::Punctuator(Punctuator::Minus) => {
                self.parse_negative_numeric_literal_pattern(state)
            }
            RawToken::Identifier => self.parse_pattern_beginning_with_identifier(state),
            RawToken::Punctuator(Punctuator::OpenBracket) => self.parse_list_pattern(state),
            RawToken::Punctuator(Punctuator::OpenParent) => {
                self.parse_grouped_or_tuple_pattern(state)
            }
            _ => {
                state.diagnostics.add_diagnostic(UnexpectedToken::new(
                    state.current_token.location.end,
                    state.next_token,
                    "pattern",
                ));
                None
            }
        }
    }
}
