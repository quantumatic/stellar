use crate::{
    error::{expected, ParseError, ParseResult},
    macros::parse_list,
    path::PathParser,
    Cursor, OptionalParser, Parse,
};
use ry_ast::{
    span::{At, Span, Spanned},
    token::RawToken,
    GenericParameter, Token, Type, WhereClause, WhereClauseItem,
};

pub(crate) struct TypeParser;

struct ArrayTypeParser;

struct PrimaryTypeParser;

struct TupleTypeParser;

struct FunctionTypeParser;

pub(crate) struct GenericParametersParser;

pub(crate) struct GenericArgumentsParser;

pub(crate) struct WhereClauseParser;

impl Parse for TypeParser {
    type Output = Spanned<Type>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        match cursor.next.unwrap() {
            RawToken::Identifier(..) => PrimaryTypeParser.parse_with(cursor),
            Token!['['] => ArrayTypeParser.parse_with(cursor),
            Token![#] => TupleTypeParser.parse_with(cursor),
            Token!['('] => FunctionTypeParser.parse_with(cursor),
            _ => Err(ParseError::unexpected_token(
                cursor.next.clone(),
                expected!("identifier", Token!['['], Token![#], Token!['(']),
                "type",
            )),
        }
    }
}

impl Parse for ArrayTypeParser {
    type Output = Spanned<Type>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.next_token();
        let start = cursor.current.span().start();

        let element_type = TypeParser.parse_with(cursor)?;

        cursor.consume(Token![']'], "array type")?;

        Ok(Type::Array {
            element_type: Box::new(element_type),
        }
        .at(Span::new(
            start,
            cursor.current.span().end(),
            cursor.file_id,
        )))
    }
}

impl Parse for TupleTypeParser {
    type Output = Spanned<Type>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.next_token(); // `#`
        let start = cursor.current.span().start();

        cursor.consume(Token!['('], "tuple type")?;

        let element_types = parse_list!(cursor, "tuple type", Token![')'], || {
            TypeParser.parse_with(cursor)
        });

        cursor.next_token(); // `)`

        Ok(Type::Tuple { element_types }.at(Span::new(
            start,
            cursor.current.span().end(),
            cursor.file_id,
        )))
    }
}

impl Parse for FunctionTypeParser {
    type Output = Spanned<Type>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
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

        Ok(Type::Function {
            parameter_types,
            return_type,
        }
        .at(Span::new(
            start,
            cursor.current.span().end(),
            cursor.file_id,
        )))
    }
}

impl OptionalParser for GenericParametersParser {
    type Output = Vec<GenericParameter>;

    fn optionally_parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        if *cursor.next.unwrap() != Token!['['] {
            return Ok(vec![]);
        }

        cursor.next_token();

        let result = parse_list!(
            cursor,
            "generics",
            Token![']'],
            || -> ParseResult<GenericParameter> {
                let name = cursor.consume_identifier("generic name")?;

                let constraint = if *cursor.next.unwrap() == Token![:] {
                    cursor.next_token();
                    Some(TypeParser.parse_with(cursor)?)
                } else {
                    None
                };

                Ok(GenericParameter { name, constraint })
            }
        );

        cursor.next_token();

        Ok(result)
    }
}

impl Parse for PrimaryTypeParser {
    type Output = Spanned<Type>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        let start = cursor.next.span().start();
        let path = PathParser.parse_with(cursor)?;
        let generic_arguments = GenericArgumentsParser.optionally_parse_with(cursor)?;

        Ok(Type::Primary {
            path,
            generic_arguments,
        }
        .at(Span::new(
            start,
            cursor.current.span().end(),
            cursor.file_id,
        )))
    }
}

impl OptionalParser for GenericArgumentsParser {
    type Output = Vec<Spanned<Type>>;

    fn optionally_parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        if *cursor.next.unwrap() != Token!['['] {
            return Ok(vec![]);
        }

        self.parse_with(cursor)
    }
}

impl Parse for GenericArgumentsParser {
    type Output = Vec<Spanned<Type>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.next_token();

        let result = parse_list!(cursor, "type annotations", Token![']'], || {
            TypeParser.parse_with(cursor)
        });

        cursor.next_token();

        Ok(result)
    }
}

impl OptionalParser for WhereClauseParser {
    type Output = WhereClause;

    fn optionally_parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        if *cursor.next.unwrap() != Token![where] {
            return Ok(vec![]);
        }

        cursor.next_token();

        Ok(parse_list!(
            cursor,
            "where clause",
            Token!['{'] | Token![;],
            || -> ParseResult<WhereClauseItem> {
                let r#type = TypeParser.parse_with(cursor)?;

                cursor.consume(Token![:], "where clause")?;

                let constraint = TypeParser.parse_with(cursor)?;

                Ok(WhereClauseItem { r#type, constraint })
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
