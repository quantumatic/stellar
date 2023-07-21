use ry_ast::{token::RawToken, Path, Pattern, StructFieldPattern, Token};

use crate::{
    diagnostics::UnexpectedTokenDiagnostic, expected, literal::LiteralParser, macros::parse_list,
    path::PathParser, Parse, ParseState,
};

pub(crate) struct PatternParser;

struct PatternExceptOrParser;

struct ArrayPatternParser;

struct GroupedOrTuplePatternParser;

struct StructPatternParser {
    pub(crate) path: Path,
}

struct TupleLikePatternParser {
    pub(crate) path: Path,
}

impl Parse for PatternParser {
    type Output = Option<Pattern>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let left = PatternExceptOrParser.parse(state)?;

        if state.next_token.raw == Token![|] {
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

impl Parse for PatternExceptOrParser {
    type Output = Option<Pattern>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        match state.next_token.raw {
            RawToken::StringLiteral
            | RawToken::CharLiteral
            | RawToken::IntegerLiteral
            | RawToken::FloatLiteral
            | RawToken::TrueBoolLiteral
            | RawToken::FalseBoolLiteral => Some(Pattern::Literal(LiteralParser.parse(state)?)),
            RawToken::Identifier => {
                let path = PathParser.parse(state)?;

                match state.next_token.raw {
                    Token!['{'] => {
                        return StructPatternParser { path }.parse(state);
                    }
                    Token!['('] => {
                        return TupleLikePatternParser { path }.parse(state);
                    }
                    _ => {}
                };

                // If it is only 1 identifier
                if path.identifiers.len() == 1 {
                    let identifier = path.identifiers.first().expect(
                        "Cannot get first identifier in path when parsing identifier pattern",
                    );

                    let pattern = if state.next_token.raw == Token![@] {
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
            Token!['['] => ArrayPatternParser.parse(state),
            Token![..] => {
                state.advance();
                Some(Pattern::Rest {
                    location: state.next_token.location,
                })
            }
            Token!['('] => GroupedOrTuplePatternParser.parse(state),
            _ => {
                state.add_diagnostic(UnexpectedTokenDiagnostic::new(
                    state.next_token,
                    expected!(
                        "integer literal",
                        "float literal",
                        "string literal",
                        "char literal",
                        "boolean literal",
                        Token!['['],
                        "identifier",
                        Token![if],
                        Token![while]
                    ),
                    "expression",
                ));
                None
            }
        }
    }
}

impl Parse for StructPatternParser {
    type Output = Option<Pattern>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance(); // `{`

        let fields = parse_list!(state, "struct pattern", Token!['}'], {
            if state.next_token.raw == Token![..] {
                state.advance();

                Some(StructFieldPattern::Rest {
                    location: state.current_token.location,
                })
            } else {
                let field_name = state.consume_identifier("struct pattern")?;
                let value_pattern = if state.next_token.raw == Token![:] {
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
        });

        state.advance();

        Some(Pattern::Struct {
            location: state.location_from(self.path.location.start),
            path: self.path,
            fields,
        })
    }
}

impl Parse for ArrayPatternParser {
    type Output = Option<Pattern>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.location.start;
        state.advance(); // `[`

        let inner_patterns = parse_list!(state, "array pattern", Token![']'], {
            PatternParser.parse(state)
        });

        state.advance(); // `]`

        Some(Pattern::List {
            location: state.location_from(start),
            inner_patterns,
        })
    }
}

impl Parse for TupleLikePatternParser {
    type Output = Option<Pattern>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance(); // `(`

        let inner_patterns = parse_list!(state, "enum item tuple pattern", Token![')'], {
            PatternParser.parse(state)
        });

        state.advance(); // `)`

        Some(Pattern::TupleLike {
            location: state.location_from(self.path.location.start),
            path: self.path,
            inner_patterns,
        })
    }
}

impl Parse for GroupedOrTuplePatternParser {
    type Output = Option<Pattern>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.location.start;
        state.advance();

        let elements = parse_list!(state, "parenthesized or tuple pattern", Token![')'], {
            PatternParser.parse(state)
        });

        state.advance(); // `)`

        let location = state.location_from(start);

        let mut elements = elements.into_iter();

        match (elements.next(), elements.next()) {
            (Some(element), None) => {
                if state
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
