use ry_ast::{
    precedence::Precedence,
    span::{At, Span, Spanned},
    token::RawToken,
    Expression, MatchExpressionUnit, StructExpressionUnit, Token,
};

use crate::{
    error::{expected, ParseError, ParseResult},
    literal::LiteralParser,
    macros::{binop_pattern, parse_list, postfixop_pattern, prefixop_pattern},
    pattern::PatternParser,
    r#type::{GenericArgumentsParser, TypeParser},
    statement::StatementsBlockParser,
    Cursor, Parse,
};

#[derive(Default)]
pub(crate) struct ExpressionParser {
    pub(crate) precedence: Precedence,
}

struct WhileExpressionParser;

struct MatchExpressionParser;

struct MatchExpressionBlockParser;

struct MatchExpressionUnitParser;

struct PrimaryExpressionParser;

struct GenericArgumentsExpressionParser {
    pub(crate) left: Spanned<Expression>,
}

struct PropertyAccessExpressionParser {
    pub(crate) left: Spanned<Expression>,
}

struct PrefixExpressionParser;

struct PostfixExpressionParser {
    pub(crate) left: Spanned<Expression>,
}

struct CastExpressionParser {
    pub(crate) left: Spanned<Expression>,
}

struct IfExpressionParser;

struct ParenthesizedExpressionParser;

struct CallExpressionParser {
    pub(crate) left: Spanned<Expression>,
}

struct BinaryExpressionParser {
    pub(crate) left: Spanned<Expression>,
}

struct ArrayLiteralExpressionParser;

struct TupleExpressionParser;

struct StructExpressionParser {
    pub(crate) left: Spanned<Expression>,
}

struct StructExpressionUnitParser;

struct LetExpressionParser;

struct StatementsBlockExpressionParser;

impl Parse for ExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        let mut left = PrimaryExpressionParser.parse_with(cursor)?;

        while self.precedence < cursor.next.unwrap().to_precedence() {
            left = match cursor.next.unwrap() {
                binop_pattern!() => BinaryExpressionParser { left }.parse_with(cursor)?,
                Token!['('] => CallExpressionParser { left }.parse_with(cursor)?,
                Token![.] => PropertyAccessExpressionParser { left }.parse_with(cursor)?,
                Token!['['] => GenericArgumentsExpressionParser { left }.parse_with(cursor)?,
                postfixop_pattern!() => PostfixExpressionParser { left }.parse_with(cursor)?,
                Token![as] => CastExpressionParser { left }.parse_with(cursor)?,
                Token!['{'] => StructExpressionParser { left }.parse_with(cursor)?,
                _ => break,
            };
        }

        Ok(left)
    }
}

impl Parse for WhileExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.next_token(); // `while`
        let start = cursor.current.span().start();

        let condition = ExpressionParser::default().parse_with(cursor)?;
        cursor.consume(Token![do], "while expression")?;

        let body = StatementsBlockParser.parse_with(cursor)?;

        Ok(Expression::While {
            condition: Box::new(condition),
            body,
        }
        .at(Span::new(
            start,
            cursor.current.span().end(),
            cursor.file_id,
        )))
    }
}

impl Parse for MatchExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.next_token(); // `match`

        let expression = ExpressionParser::default().parse_with(cursor)?;
        cursor.consume(Token![with], "match expression")?;

        let block = MatchExpressionBlockParser.parse_with(cursor)?;

        let span = Span::new(
            expression.span().start(),
            cursor.current.span().end(),
            cursor.file_id,
        );
        Ok(Expression::Match {
            expression: Box::new(expression),
            block,
        }
        .at(span))
    }
}

impl Parse for MatchExpressionBlockParser {
    type Output = Vec<MatchExpressionUnit>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.consume(Token!['{'], "match expression block")?;

        let units = parse_list!(cursor, "match expression block", Token!['}'], || {
            MatchExpressionUnitParser.parse_with(cursor)
        });

        cursor.next_token(); // `}`

        Ok(units)
    }
}

impl Parse for MatchExpressionUnitParser {
    type Output = MatchExpressionUnit;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        let left = PatternParser.parse_with(cursor)?;
        cursor.consume(Token![then], "match expression unit")?;

        let right = ExpressionParser::default().parse_with(cursor)?;

        Ok(MatchExpressionUnit { left, right })
    }
}

impl Parse for PrimaryExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        match *cursor.next.unwrap() {
            RawToken::IntegerLiteral
            | RawToken::FloatLiteral
            | RawToken::StringLiteral
            | RawToken::CharLiteral
            | Token![true]
            | Token![false] => {
                let literal = LiteralParser.parse_with(cursor)?;
                let span = literal.span();

                Ok(Expression::Literal(literal.take()).at(span))
            }
            RawToken::Identifier(symbol) => {
                cursor.next_token();
                Ok(Expression::Identifier(symbol).at(cursor.current.span()))
            }
            prefixop_pattern!() => PrefixExpressionParser.parse_with(cursor),
            Token!['('] => ParenthesizedExpressionParser.parse_with(cursor),
            Token!['['] => ArrayLiteralExpressionParser.parse_with(cursor),
            Token!['{'] => StatementsBlockExpressionParser.parse_with(cursor),
            Token![#] => TupleExpressionParser.parse_with(cursor),
            Token![let] => LetExpressionParser.parse_with(cursor),
            Token![if] => IfExpressionParser.parse_with(cursor),
            Token![match] => MatchExpressionParser.parse_with(cursor),
            Token![while] => WhileExpressionParser.parse_with(cursor),
            _ => Err(ParseError::unexpected_token(
                cursor.next.clone(),
                expected!(
                    "integer literal",
                    "float literal",
                    "string literal",
                    "char literal",
                    "boolean literal",
                    Token![#],
                    Token!['('],
                    Token!['{'],
                    Token!['['],
                    "identifier",
                    Token![if],
                    Token![while],
                    Token![match]
                ),
                "expression",
            )),
        }
    }
}

impl Parse for GenericArgumentsExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        let arguments = GenericArgumentsParser.parse_with(cursor)?;

        let span = Span::new(
            self.left.span().start(),
            cursor.current.span().end(),
            cursor.file_id,
        );

        Ok(Expression::GenericArguments {
            left: Box::new(self.left),
            arguments,
        }
        .at(span))
    }
}

impl Parse for PropertyAccessExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.next_token(); // `.`

        let start = self.left.span().start();

        Ok(Expression::Property {
            left: Box::new(self.left),
            right: cursor.consume_identifier("property")?,
        }
        .at(Span::new(
            start,
            cursor.current.span().end(),
            cursor.file_id,
        )))
    }
}

impl Parse for PrefixExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        let operator = cursor.next.clone();
        cursor.next_token();

        let inner = ExpressionParser {
            precedence: Precedence::Unary,
        }
        .parse_with(cursor)?;

        let span = Span::new(operator.span().start(), inner.span().end(), cursor.file_id);

        Ok(Expression::Unary {
            inner: Box::new(inner),
            operator,
            postfix: false,
        }
        .at(span))
    }
}

impl Parse for PostfixExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        let start = self.left.span().start();

        cursor.next_token();

        Ok(Expression::Unary {
            inner: Box::new(self.left),
            operator: cursor.current.clone(),
            postfix: true,
        }
        .at(Span::new(
            start,
            cursor.current.span().end(),
            cursor.file_id,
        )))
    }
}

impl Parse for ParenthesizedExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.next_token();
        let start = cursor.current.span().start();

        let inner = ExpressionParser {
            precedence: Precedence::Lowest,
        }
        .parse_with(cursor)?;

        cursor.consume(Token![')'], "parenthesized expression")?;

        Ok(Expression::Parenthesized {
            inner: Box::new(inner),
        }
        .at(Span::new(
            start,
            cursor.current.span().end(),
            cursor.file_id,
        )))
    }
}

impl Parse for IfExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.next_token(); // `if`

        let start = cursor.current.span().start();

        let condition = ExpressionParser::default().parse_with(cursor)?;
        cursor.consume(Token![then], "if expression")?;

        let block = StatementsBlockParser.parse_with(cursor)?;

        let mut if_blocks = vec![(condition, block)];

        let mut r#else = None;

        while cursor.next.unwrap() == &Token![else] {
            cursor.next_token();

            if cursor.next.unwrap() != &Token![if] {
                r#else = Some(StatementsBlockParser.parse_with(cursor)?);
                break;
            }

            cursor.next_token();

            let condition = ExpressionParser::default().parse_with(cursor)?;
            cursor.consume(Token![then], "if expression")?;

            let block = StatementsBlockParser.parse_with(cursor)?;
            if_blocks.push((condition, block));
        }

        Ok(Expression::If { if_blocks, r#else }.at(Span::new(
            start,
            cursor.current.span().end(),
            cursor.file_id,
        )))
    }
}

impl Parse for CastExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        let start = self.left.span().start();

        cursor.next_token();

        let right = TypeParser.parse_with(cursor)?;

        Ok(Expression::As {
            left: Box::new(self.left),
            right,
        }
        .at(Span::new(
            start,
            cursor.current.span().end(),
            cursor.file_id,
        )))
    }
}

impl Parse for CallExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        let start = self.left.span().start();

        cursor.next_token(); // `(`

        let arguments = parse_list!(cursor, "call arguments list", Token![')'], || {
            ExpressionParser {
                precedence: Precedence::Lowest,
            }
            .parse_with(cursor)
        });

        cursor.next_token();

        Ok(Expression::Call {
            left: Box::new(self.left),
            arguments,
        }
        .at(Span::new(
            start,
            cursor.current.span().end(),
            cursor.file_id,
        )))
    }
}

impl Parse for BinaryExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        let start = self.left.span().start();

        let operator = cursor.next.clone();
        let precedence = cursor.next.unwrap().to_precedence();

        cursor.next_token();

        let right = ExpressionParser { precedence }.parse_with(cursor)?;

        Ok(Expression::Binary {
            left: Box::new(self.left),
            right: Box::new(right),
            operator,
        }
        .at(Span::new(
            start,
            cursor.current.span().end(),
            cursor.file_id,
        )))
    }
}

impl Parse for ArrayLiteralExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.next_token();

        let start = cursor.next.span().start();

        let elements = parse_list!(cursor, "array literal", Token![']'], || {
            ExpressionParser::default().parse_with(cursor)
        });

        cursor.next_token();

        Ok(Expression::Array { elements }.at(Span::new(
            start,
            cursor.current.span().end(),
            cursor.file_id,
        )))
    }
}

impl Parse for TupleExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.next_token(); // `#`

        let start = cursor.current.span().start();

        cursor.consume(Token!['('], "tuple expression")?;

        let elements = parse_list!(cursor, "tuple expression", Token![')'], || {
            ExpressionParser::default().parse_with(cursor)
        });

        cursor.next_token(); // `)`

        Ok(Expression::Tuple { elements }.at(Span::new(
            start,
            cursor.current.span().end(),
            cursor.file_id,
        )))
    }
}

impl Parse for StructExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.next_token(); // `{`

        let fields = parse_list!(cursor, "struct expression", Token!['}'], || {
            StructExpressionUnitParser.parse_with(cursor)
        });

        cursor.next_token(); // `}`

        let span = Span::new(
            self.left.span().start(),
            cursor.current.span().end(),
            cursor.file_id,
        );

        Ok(Expression::Struct {
            left: Box::new(self.left),
            fields,
        }
        .at(span))
    }
}

impl Parse for StructExpressionUnitParser {
    type Output = StructExpressionUnit;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        let name = cursor.consume_identifier("struct field")?;

        let value = if cursor.next.unwrap() == &Token![:] {
            cursor.next_token();
            Some(ExpressionParser::default().parse_with(cursor)?)
        } else {
            None
        };

        Ok(StructExpressionUnit { name, value })
    }
}

impl Parse for LetExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        cursor.next_token(); // `let`

        let left = PatternParser.parse_with(cursor)?;

        cursor.consume(Token![=], "let expression")?;

        let right = Box::new(ExpressionParser::default().parse_with(cursor)?);

        let span = Span::new(left.span().start(), right.span().end(), cursor.file_id);

        Ok(Expression::Let { left, right }.at(span))
    }
}

impl Parse for StatementsBlockExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> ParseResult<Self::Output> {
        let start = cursor.next.span().start();

        let block = StatementsBlockParser.parse_with(cursor)?;
        let end = cursor.current.span().end();

        Ok(Expression::StatementsBlock(block).at(Span::new(start, end, cursor.file_id)))
    }
}

#[cfg(test)]
mod expression_tests {
    use super::ExpressionParser;
    use crate::macros::parse_test;

    parse_test!(ExpressionParser::default(), literal1, "3");
    parse_test!(ExpressionParser::default(), literal2, "\"hello\"");
    parse_test!(ExpressionParser::default(), literal3, "true");
    parse_test!(ExpressionParser::default(), array, "[1, 2, \"3\".into()]");
    parse_test!(ExpressionParser::default(), tuple, "#(1, 3.2, \"hello\")");
    parse_test!(
        ExpressionParser::default(),
        binary,
        "!(1 + 2) + 3 / (3 + a.unwrap_or(0) * 4)"
    );
    parse_test!(ExpressionParser::default(), cast, "1 as f32");
    parse_test!(ExpressionParser::default(), call, "l(2 * b() + 2).a()");
    parse_test!(
        ExpressionParser::default(),
        call_with_generics,
        "sizeof[i32]()"
    );
    parse_test!(
        ExpressionParser::default(),
        ifelse,
        "if false then { 2.3 } else if false then { 5 as f32 } else { 2.0 }"
    );
    parse_test!(
        ExpressionParser::default(),
        r#while,
        "while true do { print(\"hello\"); }"
    );
    parse_test!(
        ExpressionParser::default(),
        postfix,
        "Some(a().unwrap_or(0) + b()?)"
    );
    parse_test!(
        ExpressionParser::default(),
        r#struct,
        "Person { age: 3, name }"
    );
    parse_test!(ExpressionParser::default(), let1, "let a = 3;");
    parse_test!(
        ExpressionParser::default(),
        let2,
        "let Person { name, age } = get_person();"
    );
    parse_test!(
        ExpressionParser::default(),
        r#match,
        "match Some(3) with { Some(a) then println(a), .. then {} }"
    );
}
