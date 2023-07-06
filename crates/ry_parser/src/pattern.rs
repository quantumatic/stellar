use crate::{literal::LiteralParser, macros::parse_list, path::PathParser, Parse, ParseState};
use ry_ast::{token::RawToken, Path, Pattern, StructFieldPattern, Token};
use ry_diagnostics::{expected, parser::ParseDiagnostic, BuildDiagnostic};
use ry_span::span::SpanIndex;

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
                span: state.make_span(left.span().start, right.span().end),
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
                        span: state.make_span(
                            path.span.start,
                            match pattern {
                                Some(ref pattern) => pattern.span().end,
                                None => path.span.end,
                            },
                        ),
                        identifier: *identifier,
                        pattern,
                    })
                } else {
                    Some(Pattern::Path {
                        span: path.span,
                        path,
                    })
                }
            }
            Token!['['] => ArrayPatternParser.parse(state),
            Token![..] => {
                state.advance();
                Some(Pattern::Rest {
                    span: state.next_token.span,
                })
            }
            Token!['('] => GroupedOrTuplePatternParser.parse(state),
            _ => {
                state.diagnostics.push(
                    ParseDiagnostic::UnexpectedTokenError {
                        got: state.next_token,
                        expected: expected!(
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
                        node: "expression".into(),
                    }
                    .build(),
                );
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
                    span: state.current_token.span,
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
                    span: state.span_from(field_name.span.start),
                    field_name,
                    value_pattern,
                })
            }
        });

        state.advance();

        Some(Pattern::Struct {
            span: state.span_from(self.path.span.start),
            path: self.path,
            fields,
        })
    }
}

impl Parse for ArrayPatternParser {
    type Output = Option<Pattern>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.span.start;
        state.advance(); // `[`

        let inner_patterns = parse_list!(state, "array pattern", Token![']'], {
            PatternParser.parse(state)
        });

        state.advance(); // `]`

        Some(Pattern::List {
            span: state.span_from(start),
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
            span: state.span_from(self.path.span.start),
            path: self.path,
            inner_patterns,
        })
    }
}

impl Parse for GroupedOrTuplePatternParser {
    type Output = Option<Pattern>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.span.start;
        state.advance();

        let elements = parse_list!(state, "parenthesized or tuple pattern", Token![')'], {
            PatternParser.parse(state)
        });

        state.advance(); // `)`

        let span = state.span_from(start);

        let mut elements = elements.into_iter();

        match (elements.next(), elements.next()) {
            (Some(element), None) => {
                if state
                    .file
                    .source
                    .index(state.span_from(element.span().end))
                    .contains(',')
                {
                    Some(Pattern::Tuple {
                        span,
                        elements: vec![element],
                    })
                } else {
                    Some(Pattern::Grouped {
                        span,
                        inner: Box::from(element),
                    })
                }
            }
            (None, None) => Some(Pattern::Tuple {
                span,
                elements: vec![],
            }),
            (Some(previous), Some(next)) => {
                let mut new_elements = vec![];
                new_elements.push(previous);
                new_elements.push(next);

                new_elements.append(&mut elements.collect::<Vec<_>>());

                Some(Pattern::Tuple {
                    span,
                    elements: new_elements,
                })
            }
            _ => unreachable!(),
        }
    }
}
