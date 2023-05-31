use crate::{
    error::{expected, ParseError, ParseResult},
    literal::LiteralParser,
    macros::parse_list,
    path::PathParser,
    Cursor, Parse,
};
use ry_ast::{
    span::{At, Span, Spanned},
    token::RawToken,
    Path, Pattern, Token,
};

pub(crate) struct PatternParser;

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
    type Output = Spanned<Pattern>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        match cursor.next.unwrap() {
            RawToken::StringLiteral
            | RawToken::CharLiteral
            | RawToken::IntegerLiteral
            | RawToken::FloatLiteral
            | RawToken::TrueBoolLiteral
            | RawToken::FalseBoolLiteral => {
                let literal = LiteralParser.parse_with(cursor)?;
                let span = literal.span();

                Ok(Pattern::Literal(literal).at(span))
            }
            RawToken::Identifier(..) => {
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
                    let identifier = path
                        .unwrap()
                        .get(0)
                        .expect(
                            "Cannot get first identifier in path when parsing identifier pattern",
                        )
                        .to_owned();

                    let pattern = if cursor.next.unwrap() == &Token![@] {
                        cursor.next_token();
                        Some(Box::new(PatternParser.parse_with(cursor)?))
                    } else {
                        None
                    };

                    let span = Span::new(
                        identifier.span().start(),
                        match pattern {
                            Some(ref pattern) => pattern.span().end(),
                            None => identifier.span().end(),
                        },
                        cursor.file_id,
                    );

                    Ok(Pattern::Identifier {
                        identifier,
                        pattern,
                    }
                    .at(span))
                } else {
                    let span = path.span();
                    Ok(Pattern::Path { path }.at(span))
                }
            }
            Token!['['] => ArrayPatternParser.parse_with(cursor),
            Token![..] => {
                cursor.next_token();
                Ok(Pattern::Rest.at(cursor.next.span()))
            }
            Token!['('] => GroupedPatternParser.parse_with(cursor),
            Token![#] => TuplePatternParser.parse_with(cursor),
            _ => Err(ParseError::unexpected_token(
                cursor.next.clone(),
                expected!(
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
                "expression",
            )),
        }
    }
}

impl Parse for StructPatternParser {
    type Output = Spanned<Pattern>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.next_token(); // `{`

        let fields = parse_list!(cursor, "struct pattern", Token!['}'], || {
            let member_name = cursor.consume_identifier("struct pattern")?;
            let pattern = if cursor.next.unwrap() == &Token![:] {
                cursor.next_token();

                Some(PatternParser.parse_with(cursor)?)
            } else {
                None
            };

            Ok::<(Spanned<usize>, Option<Spanned<Pattern>>), ParseError>((member_name, pattern))
        });

        cursor.next_token();

        let span = Span::new(
            self.r#struct.span().start(),
            cursor.current.span().end(),
            cursor.file_id,
        );

        Ok(Pattern::Struct {
            r#struct: self.r#struct,
            fields,
        }
        .at(span))
    }
}

impl Parse for ArrayPatternParser {
    type Output = Spanned<Pattern>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.next_token(); // `[`

        let start = cursor.current.span().start();

        let inner_patterns = parse_list!(cursor, "array pattern", Token![']'], || {
            PatternParser.parse_with(cursor)
        });

        cursor.next_token(); // `]`

        Ok(Pattern::Array { inner_patterns }.at(Span::new(
            start,
            cursor.current.span().end(),
            cursor.file_id,
        )))
    }
}

impl Parse for TuplePatternParser {
    type Output = Spanned<Pattern>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.next_token(); // `#`
        let start = cursor.current.span().start();

        cursor.consume(Token!['('], "tuple pattern")?;

        let inner_patterns = parse_list!(cursor, "tuple pattern", Token![')'], || {
            PatternParser.parse_with(cursor)
        });

        cursor.next_token(); // `)`

        Ok(Pattern::Tuple { inner_patterns }.at(Span::new(
            start,
            cursor.current.span().end(),
            cursor.file_id,
        )))
    }
}

impl Parse for EnumItemTuplePatternParser {
    type Output = Spanned<Pattern>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
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

        Ok(Pattern::EnumItemTuple {
            r#enum: self.r#enum,
            inner_patterns,
        }
        .at(span))
    }
}

impl Parse for GroupedPatternParser {
    type Output = Spanned<Pattern>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.next_token(); // `(`

        let start = cursor.current.span().start();

        let inner = Box::new(PatternParser.parse_with(cursor)?);

        cursor.consume(Token![')'], "grouped pattern")?;

        Ok(Pattern::Grouped { inner }.at(Span::new(
            start,
            cursor.current.span().end(),
            cursor.file_id,
        )))
    }
}

#[cfg(test)]
mod pattern_tests {
    use crate::{macros::parse_test, pattern::PatternParser};

    parse_test!(PatternParser, rest_pattern, "..");
    parse_test!(PatternParser, identifier_pattern, "test");
    parse_test!(PatternParser, path_pattern, "a.b.c");
    parse_test!(PatternParser, identifier_pattern2, "test @ 1");
    parse_test!(PatternParser, identifier_pattern3, "test @ [1, ..]");
    parse_test!(PatternParser, struct_pattern, "test { a: 2, b, c: d }");
    parse_test!(PatternParser, tuple_pattern, "#(a, 2, ..)");
    parse_test!(PatternParser, enum_item_tuple_pattern, "Some(a)");
    parse_test!(PatternParser, grouped_pattern, "(Some(a))");
}
