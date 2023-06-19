use crate::{literal::LiteralParser, macros::parse_list, path::PathParser, Cursor, Parse};
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

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let left = PatternExceptOrParser.parse_with(cursor)?;

        if cursor.next.raw == Token![|] {
            cursor.next_token();

            let right = Self.parse_with(cursor)?;

            Some(Pattern::Or {
                span: Span::new(left.span().start(), right.span().end(), cursor.file_id),
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

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        match cursor.next.raw {
            RawToken::StringLiteral
            | RawToken::CharLiteral
            | RawToken::IntegerLiteral
            | RawToken::FloatLiteral
            | RawToken::TrueBoolLiteral
            | RawToken::FalseBoolLiteral => {
                Some(Pattern::Literal(LiteralParser.parse_with(cursor)?))
            }
            RawToken::Identifier => {
                let path = PathParser.parse_with(cursor)?;

                match cursor.next.raw {
                    Token!['{'] => {
                        return StructPatternParser { r#struct: path }.parse_with(cursor);
                    }
                    Token!['('] => {
                        return TupleLikePatternParser { r#enum: path }.parse_with(cursor);
                    }
                    _ => {}
                };

                // If it is only 1 identifier
                if path.symbols.len() == 1 {
                    let identifier = path.symbols.first().expect(
                        "Cannot get first identifier in path when parsing identifier pattern",
                    );

                    let pattern = if cursor.next.raw == Token![@] {
                        cursor.next_token();
                        Some(Box::new(PatternParser.parse_with(cursor)?))
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
                            cursor.file_id,
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
            Token!['['] => ArrayPatternParser.parse_with(cursor),
            Token![..] => {
                cursor.next_token();
                Some(Pattern::Rest {
                    span: cursor.next.span,
                })
            }
            Token!['('] => GroupedPatternParser.parse_with(cursor),
            Token![#] => TuplePatternParser.parse_with(cursor),
            _ => {
                cursor.diagnostics.push(
                    ParseDiagnostic::UnexpectedTokenError {
                        got: cursor.next,
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

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token(); // `{`

        let fields = parse_list!(cursor, "struct pattern", Token!['}'], || {
            if cursor.next.raw == Token![..] {
                cursor.next_token();

                Some(StructFieldPattern::Rest {
                    span: cursor.current.span,
                })
            } else {
                let field_name = cursor.consume_identifier("struct pattern")?;
                let value_pattern = if cursor.next.raw == Token![:] {
                    cursor.next_token();

                    Some(PatternParser.parse_with(cursor)?)
                } else {
                    None
                };

                Some(StructFieldPattern::NotRest {
                    span: Span::new(
                        field_name.span.start(),
                        cursor.current.span.end(),
                        cursor.file_id,
                    ),
                    field_name,
                    value_pattern,
                })
            }
        });

        cursor.next_token();

        Some(Pattern::Struct {
            span: Span::new(
                self.r#struct.span.start(),
                cursor.current.span.end(),
                cursor.file_id,
            ),
            r#struct: self.r#struct,
            fields,
        })
    }
}

impl Parse for ArrayPatternParser {
    type Output = Option<Pattern>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let start = cursor.next.span.start();
        cursor.next_token(); // `[`

        let inner_patterns = parse_list!(cursor, "array pattern", Token![']'], || {
            PatternParser.parse_with(cursor)
        });

        cursor.next_token(); // `]`

        Some(Pattern::List {
            span: Span::new(start, cursor.current.span.end(), cursor.file_id),
            inner_patterns,
        })
    }
}

impl Parse for TuplePatternParser {
    type Output = Option<Pattern>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let start = cursor.next.span.start();
        cursor.next_token(); // `#`

        cursor.consume(Token!['('], "tuple pattern")?;

        let inner_patterns = parse_list!(cursor, "tuple pattern", Token![')'], || {
            PatternParser.parse_with(cursor)
        });

        cursor.next_token(); // `)`

        Some(Pattern::Tuple {
            span: Span::new(start, cursor.current.span.end(), cursor.file_id),
            inner_patterns,
        })
    }
}

impl Parse for TupleLikePatternParser {
    type Output = Option<Pattern>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token(); // `(`

        let inner_patterns = parse_list!(cursor, "enum item tuple pattern", Token![')'], || {
            PatternParser.parse_with(cursor)
        });

        cursor.next_token(); // `)`

        Some(Pattern::TupleLike {
            span: Span::new(
                self.r#enum.span.start(),
                cursor.current.span.end(),
                cursor.file_id,
            ),
            r#enum: self.r#enum,
            inner_patterns,
        })
    }
}

impl Parse for GroupedPatternParser {
    type Output = Option<Pattern>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let start = cursor.next.span.start();
        cursor.next_token(); // `(`

        let inner = Box::new(PatternParser.parse_with(cursor)?);

        cursor.consume(Token![')'], "grouped pattern")?;

        Some(Pattern::Grouped {
            span: Span::new(start, cursor.current.span.end(), cursor.file_id),
            inner,
        })
    }
}
