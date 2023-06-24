use crate::{literal::LiteralParser, macros::parse_list, path::PathParser, Parse, TokenIterator};
use ry_ast::{token::RawToken, Path, Pattern, StructFieldPattern, Token};
use ry_diagnostics::{expected, parser::ParseDiagnostic, Report};
use ry_source_file::span::Span;

pub(crate) struct PatternParser;

struct PatternExceptOrParser;

struct ArrayPatternParser;

struct GroupedPatternParser;

struct TuplePatternParser;

struct StructPatternParser {
    pub(crate) r#struct: Path,
}

struct TupleLikePatternParser {
    pub(crate) r#enum: Path,
}

impl Parse for PatternParser {
    type Output = Option<Pattern>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let left = PatternExceptOrParser.parse_using(iterator)?;

        if iterator.next_token.raw == Token![|] {
            iterator.advance();

            let right = Self.parse_using(iterator)?;

            Some(Pattern::Or {
                span: Span::new(left.span().start(), right.span().end(), iterator.file_id),
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

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        match iterator.next_token.raw {
            RawToken::StringLiteral
            | RawToken::CharLiteral
            | RawToken::IntegerLiteral
            | RawToken::FloatLiteral
            | RawToken::TrueBoolLiteral
            | RawToken::FalseBoolLiteral => {
                Some(Pattern::Literal(LiteralParser.parse_using(iterator)?))
            }
            RawToken::Identifier => {
                let path = PathParser.parse_using(iterator)?;

                match iterator.next_token.raw {
                    Token!['{'] => {
                        return StructPatternParser { r#struct: path }.parse_using(iterator);
                    }
                    Token!['('] => {
                        return TupleLikePatternParser { r#enum: path }.parse_using(iterator);
                    }
                    _ => {}
                };

                // If it is only 1 identifier
                if path.identifiers.len() == 1 {
                    let identifier = path.identifiers.first().expect(
                        "Cannot get first identifier in path when parsing identifier pattern",
                    );

                    let pattern = if iterator.next_token.raw == Token![@] {
                        iterator.advance();
                        Some(Box::new(PatternParser.parse_using(iterator)?))
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
                            iterator.file_id,
                        ),
                        identifier: *identifier,
                        ty: None,
                        pattern,
                    })
                } else {
                    Some(Pattern::Path {
                        span: path.span,
                        path,
                    })
                }
            }
            Token!['['] => ArrayPatternParser.parse_using(iterator),
            Token![..] => {
                iterator.advance();
                Some(Pattern::Rest {
                    span: iterator.next_token.span,
                })
            }
            Token!['('] => GroupedPatternParser.parse_using(iterator),
            Token![#] => TuplePatternParser.parse_using(iterator),
            _ => {
                iterator.diagnostics.push(
                    ParseDiagnostic::UnexpectedTokenError {
                        got: iterator.next_token,
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

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        iterator.advance(); // `{`

        let fields = parse_list!(iterator, "struct pattern", Token!['}'], {
            if iterator.next_token.raw == Token![..] {
                iterator.advance();

                Some(StructFieldPattern::Rest {
                    span: iterator.current_token.span,
                })
            } else {
                let field_name = iterator.consume_identifier("struct pattern")?;
                let value_pattern = if iterator.next_token.raw == Token![:] {
                    iterator.advance();

                    Some(PatternParser.parse_using(iterator)?)
                } else {
                    None
                };

                Some(StructFieldPattern::NotRest {
                    span: Span::new(
                        field_name.span.start(),
                        iterator.current_token.span.end(),
                        iterator.file_id,
                    ),
                    field_name,
                    value_pattern,
                })
            }
        });

        iterator.advance();

        Some(Pattern::Struct {
            span: Span::new(
                self.r#struct.span.start(),
                iterator.current_token.span.end(),
                iterator.file_id,
            ),
            r#struct: self.r#struct,
            fields,
        })
    }
}

impl Parse for ArrayPatternParser {
    type Output = Option<Pattern>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let start = iterator.next_token.span.start();
        iterator.advance(); // `[`

        let inner_patterns = parse_list!(iterator, "array pattern", Token![']'], {
            PatternParser.parse_using(iterator)
        });

        iterator.advance(); // `]`

        Some(Pattern::List {
            span: Span::new(start, iterator.current_token.span.end(), iterator.file_id),
            inner_patterns,
        })
    }
}

impl Parse for TuplePatternParser {
    type Output = Option<Pattern>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let start = iterator.next_token.span.start();
        iterator.advance(); // `#`

        iterator.consume(Token!['('], "tuple pattern")?;

        let inner_patterns = parse_list!(iterator, "tuple pattern", Token![')'], {
            PatternParser.parse_using(iterator)
        });

        iterator.advance(); // `)`

        Some(Pattern::Tuple {
            span: Span::new(start, iterator.current_token.span.end(), iterator.file_id),
            inner_patterns,
        })
    }
}

impl Parse for TupleLikePatternParser {
    type Output = Option<Pattern>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        iterator.advance(); // `(`

        let inner_patterns = parse_list!(iterator, "enum item tuple pattern", Token![')'], {
            PatternParser.parse_using(iterator)
        });

        iterator.advance(); // `)`

        Some(Pattern::TupleLike {
            span: Span::new(
                self.r#enum.span.start(),
                iterator.current_token.span.end(),
                iterator.file_id,
            ),
            r#enum: self.r#enum,
            inner_patterns,
        })
    }
}

impl Parse for GroupedPatternParser {
    type Output = Option<Pattern>;

    fn parse_using(self, iterator: &mut TokenIterator<'_>) -> Self::Output {
        let start = iterator.next_token.span.start();
        iterator.advance(); // `(`

        let inner = Box::new(PatternParser.parse_using(iterator)?);

        iterator.consume(Token![')'], "grouped pattern")?;

        Some(Pattern::Grouped {
            span: Span::new(start, iterator.current_token.span.end(), iterator.file_id),
            inner,
        })
    }
}
