use crate::{literal::LiteralParser, macros::parse_list, path::PathParser, Cursor, Parse};
use ry_ast::{token::RawToken, Path, Pattern, StructFieldPattern, Token};
use ry_diagnostics::{expected, parser::ParseDiagnostic, Report};
use ry_source_file::span::{At, Span, Spanned};

pub(crate) struct PatternParser;

struct PatternExceptOrParser;

struct ArrayPatternParser;

struct GroupedPatternParser;

struct TuplePatternParser;

struct StructPatternParser {
    pub(crate) r#struct: Path,
}

struct EnumItemTuplePatternParser {
    pub(crate) r#enum: Path,
}

impl Parse for PatternParser {
    type Output = Option<Spanned<Pattern>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let left = PatternExceptOrParser.parse_with(cursor)?;

        if cursor.next.unwrap() == &Token![|] {
            cursor.next_token();

            let right = Self.parse_with(cursor)?;

            let span = Span::new(left.span().start(), right.span().end(), cursor.file_id);

            Some(
                Pattern::Or {
                    left: Box::new(left),
                    right: Box::new(right),
                }
                .at(span),
            )
        } else {
            Some(left)
        }
    }
}

impl Parse for PatternExceptOrParser {
    type Output = Option<Spanned<Pattern>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        match cursor.next.unwrap() {
            RawToken::StringLiteral
            | RawToken::CharLiteral
            | RawToken::IntegerLiteral
            | RawToken::FloatLiteral
            | RawToken::TrueBoolLiteral
            | RawToken::FalseBoolLiteral => {
                let literal = LiteralParser.parse_with(cursor)?;
                let span = literal.span();

                Some(Pattern::Literal(literal).at(span))
            }
            RawToken::Identifier => {
                let path = PathParser.parse_with(cursor)?;

                match cursor.next.unwrap() {
                    Token!['{'] => {
                        return StructPatternParser { r#struct: path }.parse_with(cursor);
                    }
                    Token!['('] => {
                        return EnumItemTuplePatternParser { r#enum: path }.parse_with(cursor);
                    }
                    _ => {}
                };

                // If it is only 1 identifier
                if path.unwrap().len() == 1 {
                    let identifier = *path.unwrap().first().expect(
                        "Cannot get first identifier in path when parsing identifier pattern",
                    );

                    let pattern = if cursor.next.unwrap() == &Token![@] {
                        cursor.next_token();
                        Some(Box::new(PatternParser.parse_with(cursor)?))
                    } else {
                        None
                    };

                    let span = Span::new(
                        path.span().start(),
                        match pattern {
                            Some(ref pattern) => pattern.span().end(),
                            None => path.span().end(),
                        },
                        cursor.file_id,
                    );

                    Some(
                        Pattern::Identifier {
                            identifier: identifier.at(path.span()),
                            ty: None,
                            pattern,
                        }
                        .at(span),
                    )
                } else {
                    let span = path.span();
                    Some(Pattern::Path { path }.at(span))
                }
            }
            Token!['['] => ArrayPatternParser.parse_with(cursor),
            Token![..] => {
                cursor.next_token();
                Some(Pattern::Rest.at(cursor.next.span()))
            }
            Token!['('] => GroupedPatternParser.parse_with(cursor),
            Token![#] => TuplePatternParser.parse_with(cursor),
            _ => {
                cursor.diagnostics.push(
                    ParseDiagnostic::UnexpectedTokenError {
                        got: cursor.next.clone(),
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
    type Output = Option<Spanned<Pattern>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token(); // `{`

        let mut rest_pattern_span = None;

        let fields = parse_list!(cursor, "struct pattern", Token!['}'], || {
            if cursor.next.unwrap() == &Token![..] {
                if let Some(previous_rest_pattern_span) = rest_pattern_span {
                    cursor.diagnostics.push(
                        ParseDiagnostic::MoreThanTwoRestPatternsInStructPatternMembersError {
                            struct_name_span: self.r#struct.span(),
                            previous_rest_pattern_span,
                            current_rest_pattern_span: cursor.next.span(),
                        }
                        .build(),
                    );
                    return None;
                }

                cursor.next_token();
                rest_pattern_span = Some(cursor.current.span());
                Some(StructFieldPattern::Rest {
                    at: cursor.current.span(),
                })
            } else {
                let field_name = cursor.consume_identifier("struct pattern")?;
                let value_pattern = if cursor.next.unwrap() == &Token![:] {
                    cursor.next_token();

                    Some(PatternParser.parse_with(cursor)?)
                } else {
                    None
                };

                let field_ty = cursor.new_unification_variable(field_name.span());

                Some(StructFieldPattern::NotRest {
                    field_name,
                    field_ty,
                    value_pattern,
                })
            }
        });

        cursor.next_token();

        let span = Span::new(
            self.r#struct.span().start(),
            cursor.current.span().end(),
            cursor.file_id,
        );

        Some(
            Pattern::Struct {
                r#struct: self.r#struct,
                fields,
            }
            .at(span),
        )
    }
}

impl Parse for ArrayPatternParser {
    type Output = Option<Spanned<Pattern>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token(); // `[`

        let start = cursor.current.span().start();

        let inner_patterns = parse_list!(cursor, "array pattern", Token![']'], || {
            PatternParser.parse_with(cursor)
        });

        cursor.next_token(); // `]`

        Some(Pattern::Array { inner_patterns }.at(Span::new(
            start,
            cursor.current.span().end(),
            cursor.file_id,
        )))
    }
}

impl Parse for TuplePatternParser {
    type Output = Option<Spanned<Pattern>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token(); // `#`
        let start = cursor.current.span().start();

        cursor.consume(Token!['('], "tuple pattern")?;

        let inner_patterns = parse_list!(cursor, "tuple pattern", Token![')'], || {
            PatternParser.parse_with(cursor)
        });

        cursor.next_token(); // `)`

        Some(Pattern::Tuple { inner_patterns }.at(Span::new(
            start,
            cursor.current.span().end(),
            cursor.file_id,
        )))
    }
}

impl Parse for EnumItemTuplePatternParser {
    type Output = Option<Spanned<Pattern>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token(); // `(`

        let inner_patterns = parse_list!(cursor, "enum item tuple pattern", Token![')'], || {
            PatternParser.parse_with(cursor)
        });

        cursor.next_token(); // `)`

        let span = Span::new(
            self.r#enum.span().start(),
            cursor.current.span().end(),
            cursor.file_id,
        );

        Some(
            Pattern::TupleLike {
                r#enum: self.r#enum,
                inner_patterns,
            }
            .at(span),
        )
    }
}

impl Parse for GroupedPatternParser {
    type Output = Option<Spanned<Pattern>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token(); // `(`

        let start = cursor.current.span().start();

        let inner = Box::new(PatternParser.parse_with(cursor)?);

        cursor.consume(Token![')'], "grouped pattern")?;

        Some(Pattern::Grouped { inner }.at(Span::new(
            start,
            cursor.current.span().end(),
            cursor.file_id,
        )))
    }
}
