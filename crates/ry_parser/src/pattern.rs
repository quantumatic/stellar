use crate::{literal::LiteralParser, macros::parse_list, path::PathParser, Parse, ParseState};
use ry_ast::{token::RawToken, Path, Pattern, StructFieldPattern, Token};
use ry_diagnostics::{expected, parser::ParseDiagnostic, BuildDiagnostic};
use ry_source_file::span::Span;

pub(crate) struct PatternParser;

struct PatternExceptOrParser;

struct ArrayPatternParser;

struct GroupedPatternParser;

struct TuplePatternParser;

struct StructPatternParser {
    pub(crate) path: Path,
}

struct TupleLikePatternParser {
    pub(crate) path: Path,
}

impl Parse for PatternParser {
    type Output = Option<Pattern>;

    fn parse(self, state: &mut ParseState<'_>) -> Self::Output {
        let left = PatternExceptOrParser.parse(state)?;

        if state.next_token.raw == Token![|] {
            state.advance();

            let right = Self.parse(state)?;

            Some(Pattern::Or {
                span: Span::new(left.span().start(), right.span().end(), state.file_id),
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

    fn parse(self, state: &mut ParseState<'_>) -> Self::Output {
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
                        span: Span::new(
                            path.span.start(),
                            match pattern {
                                Some(ref pattern) => pattern.span().end(),
                                None => path.span.end(),
                            },
                            state.file_id,
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
            Token!['('] => GroupedPatternParser.parse(state),
            Token![#] => TuplePatternParser.parse(state),
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
                            Token![#],
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

    fn parse(self, state: &mut ParseState<'_>) -> Self::Output {
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
                    span: Span::new(
                        field_name.span.start(),
                        state.current_token.span.end(),
                        state.file_id,
                    ),
                    field_name,
                    value_pattern,
                })
            }
        });

        state.advance();

        Some(Pattern::Struct {
            span: Span::new(
                self.path.span.start(),
                state.current_token.span.end(),
                state.file_id,
            ),
            path: self.path,
            fields,
        })
    }
}

impl Parse for ArrayPatternParser {
    type Output = Option<Pattern>;

    fn parse(self, state: &mut ParseState<'_>) -> Self::Output {
        let start = state.next_token.span.start();
        state.advance(); // `[`

        let inner_patterns = parse_list!(state, "array pattern", Token![']'], {
            PatternParser.parse(state)
        });

        state.advance(); // `]`

        Some(Pattern::List {
            span: Span::new(start, state.current_token.span.end(), state.file_id),
            inner_patterns,
        })
    }
}

impl Parse for TuplePatternParser {
    type Output = Option<Pattern>;

    fn parse(self, state: &mut ParseState<'_>) -> Self::Output {
        let start = state.next_token.span.start();
        state.advance(); // `#`

        state.consume(Token!['('], "tuple pattern")?;

        let inner_patterns = parse_list!(state, "tuple pattern", Token![')'], {
            PatternParser.parse(state)
        });

        state.advance(); // `)`

        Some(Pattern::Tuple {
            span: Span::new(start, state.current_token.span.end(), state.file_id),
            inner_patterns,
        })
    }
}

impl Parse for TupleLikePatternParser {
    type Output = Option<Pattern>;

    fn parse(self, state: &mut ParseState<'_>) -> Self::Output {
        state.advance(); // `(`

        let inner_patterns = parse_list!(state, "enum item tuple pattern", Token![')'], {
            PatternParser.parse(state)
        });

        state.advance(); // `)`

        Some(Pattern::TupleLike {
            span: Span::new(
                self.path.span.start(),
                state.current_token.span.end(),
                state.file_id,
            ),
            path: self.path,
            inner_patterns,
        })
    }
}

impl Parse for GroupedPatternParser {
    type Output = Option<Pattern>;

    fn parse(self, state: &mut ParseState<'_>) -> Self::Output {
        let start = state.next_token.span.start();
        state.advance(); // `(`

        let inner = Box::new(PatternParser.parse(state)?);

        state.consume(Token![')'], "grouped pattern")?;

        Some(Pattern::Grouped {
            span: Span::new(start, state.current_token.span.end(), state.file_id),
            inner,
        })
    }
}
