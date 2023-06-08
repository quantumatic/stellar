use crate::{macros::parse_list, path::PathParser, Cursor, OptionalParser, Parse};
use ry_ast::{token::RawToken, GenericParameter, Token, Type, WhereClause, WhereClauseItem};
use ry_diagnostics::{expected, parser::ParseDiagnostic, Report};
use ry_span::{At, Span, Spanned};

pub(crate) struct TypeParser;

struct ArrayTypeParser;

struct PrimaryTypeParser;

struct TupleTypeParser;

struct FunctionTypeParser;

pub(crate) struct GenericParametersParser;

pub(crate) struct GenericArgumentsParser;

pub(crate) struct WhereClauseParser;

impl Parse for TypeParser {
    type Output = Option<Spanned<Type>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        match cursor.next.unwrap() {
            RawToken::Identifier => PrimaryTypeParser.parse_with(cursor),
            Token!['['] => ArrayTypeParser.parse_with(cursor),
            Token![#] => TupleTypeParser.parse_with(cursor),
            Token!['('] => FunctionTypeParser.parse_with(cursor),
            _ => {
                cursor.diagnostics.push(
                    ParseDiagnostic::UnexpectedTokenError {
                        got: cursor.next.clone(),
                        expected: expected!("identifier", Token!['['], Token![#], Token!['(']),
                        node: "type".to_owned(),
                    }
                    .build(),
                );

                None
            }
        }
    }
}

impl Parse for ArrayTypeParser {
    type Output = Option<Spanned<Type>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token();
        let start = cursor.current.span().start();

        let element_type = TypeParser.parse_with(cursor)?;

        if cursor.next.unwrap() == &Token![']'] {
            cursor.next_token();
        } else {
            cursor.diagnostics.push(
                ParseDiagnostic::UnexpectedTokenError {
                    got: cursor.next.clone(),
                    expected: expected!(Token![']']),
                    node: "array type".to_owned(),
                }
                .build(),
            );
        }

        Some(
            Type::Array {
                element_type: Box::new(element_type),
            }
            .at(Span::new(
                start,
                cursor.current.span().end(),
                cursor.file_id,
            )),
        )
    }
}

impl Parse for TupleTypeParser {
    type Output = Option<Spanned<Type>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token(); // `#`
        let start = cursor.current.span().start();

        cursor.consume(Token!['('], "tuple type")?;

        let element_types = parse_list!(cursor, "tuple type", Token![')'], || {
            TypeParser.parse_with(cursor)
        });

        cursor.next_token(); // `)`

        Some(Type::Tuple { element_types }.at(Span::new(
            start,
            cursor.current.span().end(),
            cursor.file_id,
        )))
    }
}

impl Parse for FunctionTypeParser {
    type Output = Option<Spanned<Type>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token(); // `(`
        let start = cursor.current.span().start();

        let parameter_types = parse_list!(
            cursor,
            "parameter types in function type",
            Token![')'],
            || { TypeParser.parse_with(cursor) }
        );

        cursor.next_token(); // `)`

        cursor.consume(Token![:], "return type of function in the function type")?;

        let return_type = Box::new(TypeParser.parse_with(cursor)?);

        Some(
            Type::Function {
                parameter_types,
                return_type,
            }
            .at(Span::new(
                start,
                cursor.current.span().end(),
                cursor.file_id,
            )),
        )
    }
}

impl OptionalParser for GenericParametersParser {
    type Output = Option<Vec<GenericParameter>>;

    fn optionally_parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        if *cursor.next.unwrap() != Token!['['] {
            return Some(vec![]);
        }

        cursor.next_token();

        let result = parse_list!(
            cursor,
            "generics",
            Token![']'],
            || -> Option<GenericParameter> {
                let name = cursor.consume_identifier("generic name")?;

                let constraint = if *cursor.next.unwrap() == Token![:] {
                    cursor.next_token();
                    Some(TypeParser.parse_with(cursor)?)
                } else {
                    None
                };

                Some(GenericParameter { name, constraint })
            }
        );

        cursor.next_token();

        Some(result)
    }
}

impl Parse for PrimaryTypeParser {
    type Output = Option<Spanned<Type>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let start = cursor.next.span().start();
        let path = PathParser.parse_with(cursor)?;
        let generic_arguments = GenericArgumentsParser.optionally_parse_with(cursor)?;

        Some(
            Type::Primary {
                path,
                generic_arguments,
            }
            .at(Span::new(
                start,
                cursor.current.span().end(),
                cursor.file_id,
            )),
        )
    }
}

impl OptionalParser for GenericArgumentsParser {
    type Output = Option<Vec<Spanned<Type>>>;

    fn optionally_parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        if *cursor.next.unwrap() != Token!['['] {
            return Some(vec![]);
        }

        self.parse_with(cursor)
    }
}

impl Parse for GenericArgumentsParser {
    type Output = Option<Vec<Spanned<Type>>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token();

        let result = parse_list!(cursor, "type annotations", Token![']'], || {
            TypeParser.parse_with(cursor)
        });

        cursor.next_token();

        Some(result)
    }
}

impl OptionalParser for WhereClauseParser {
    type Output = Option<WhereClause>;

    fn optionally_parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        if *cursor.next.unwrap() != Token![where] {
            return Some(vec![]);
        }

        cursor.next_token();

        Some(parse_list!(
            cursor,
            "where clause",
            (Token!['{']) or (Token![;]),
            || -> Option<WhereClauseItem> {
                let r#type = TypeParser.parse_with(cursor)?;

                cursor.consume(Token![:], "where clause")?;

                let constraint = TypeParser.parse_with(cursor)?;

                Some(WhereClauseItem { r#type, constraint })
            }
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::TypeParser;
    use crate::macros::parse_test;

    parse_test!(TypeParser, primary1, "i32");
    parse_test!(TypeParser, primary, "Result[T, DivisionError]");
    parse_test!(TypeParser, array, "[i32]");
    parse_test!(TypeParser, tuple, "#(i32, string, char)");
    parse_test!(TypeParser, function_type, "(i32, i32): i32");
}
