use ry_ast::{
    precedence::Precedence,
    span::{At, Span, SpanIndex, Spanned},
    token::RawToken,
    Expression, Literal, Token,
};

use crate::{
    error::{expected, ParseError, ParseResult},
    macros::{binop_pattern, parse_list, postfixop_pattern, prefixop_pattern},
    r#type::{GenericArgumentsParser, TypeParser},
    statement::StatementsBlockParser,
    Parser, ParserState,
};

#[derive(Default)]
pub(crate) struct ExpressionParser {
    pub(crate) precedence: Precedence,
}

impl Parser for ExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let mut left = PrimaryExpressionParser.parse_with(state)?;

        while self.precedence < state.next.unwrap().to_precedence() {
            left = match state.next.unwrap() {
                binop_pattern!() => BinaryExpressionParser { left }.parse_with(state)?,
                Token!['('] => CallExpressionParser { left }.parse_with(state)?,
                Token![.] => PropertyAccessExpressionParser { left }.parse_with(state)?,
                Token!['['] => GenericArgumentsExpressionParser { left }.parse_with(state)?,
                postfixop_pattern!() => PostfixExpressionParser { left }.parse_with(state)?,
                Token![as] => CastExpressionParser { left }.parse_with(state)?,
                _ => break,
            };
        }

        Ok(left)
    }
}

pub(crate) struct WhileExpressionParser;

impl Parser for WhileExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();
        let start = state.current.span().start();

        let condition = ExpressionParser::default().parse_with(state)?;
        let body = StatementsBlockParser.parse_with(state)?;

        Ok(Expression::While {
            condition: Box::new(condition),
            body,
        }
        .at(Span::new(start, state.current.span().end(), state.file_id)))
    }
}

pub(crate) struct PrimaryExpressionParser;

impl Parser for PrimaryExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        match *state.next.unwrap() {
            RawToken::IntegerLiteral => {
                state.next_token();
                match state
                    .contents
                    .index(state.current.span())
                    .replace('_', "")
                    .parse::<u64>()
                {
                    Ok(integer) => {
                        Ok(Expression::Literal(Literal::Integer(integer)).at(state.current.span()))
                    }
                    Err(..) => Err(ParseError::integer_overflow(state.current.span())),
                }
            }
            RawToken::FloatLiteral => {
                state.next_token();
                match state
                    .contents
                    .index(state.current.span())
                    .replace('_', "")
                    .parse::<f64>()
                {
                    Ok(float) => {
                        Ok(Expression::Literal(Literal::Float(float)).at(state.current.span()))
                    }
                    Err(..) => Err(ParseError::float_overflow(state.current.span())),
                }
            }
            RawToken::StringLiteral => {
                state.next_token();
                Ok(Expression::Literal(Literal::String(
                    state.contents.index(state.current.span()).to_owned(),
                ))
                .at(state.current.span()))
            }
            RawToken::CharLiteral => {
                state.next_token();
                Ok(Expression::Literal(Literal::String(
                    state.contents.index(state.current.span()).to_owned(),
                ))
                .at(state.current.span()))
            }
            Token![true] => {
                state.next_token();
                Ok(Expression::Literal(Literal::Boolean(true)).at(state.current.span()))
            }
            Token![false] => {
                state.next_token();
                Ok(Expression::Literal(Literal::Boolean(false)).at(state.current.span()))
            }
            prefixop_pattern!() => PrefixExpressionParser.parse_with(state),
            Token!['('] => ParenthesizedExpressionParser.parse_with(state),
            Token!['['] => ArrayLiteralExpressionParser.parse_with(state),
            RawToken::Identifier(identifier) => {
                state.next_token();
                Ok(Expression::Identifier(identifier).at(state.current.span()))
            }
            Token![if] => IfExpressionParser.parse_with(state),
            Token![while] => WhileExpressionParser.parse_with(state),
            _ => Err(ParseError::unexpected_token(
                state.next.clone(),
                expected!(
                    "integer literal",
                    "float literal",
                    "imaginary number literal",
                    "string literal",
                    "char literal",
                    "boolean literal",
                    Token!['('],
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

pub(crate) struct GenericArgumentsExpressionParser {
    pub(crate) left: Spanned<Expression>,
}

impl Parser for GenericArgumentsExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let arguments = GenericArgumentsParser.parse_with(state)?;

        let span = Span::new(
            self.left.span().start(),
            state.current.span().end(),
            state.file_id,
        );

        Ok(Expression::GenericArguments {
            left: Box::new(self.left),
            arguments,
        }
        .at(span))
    }
}

pub(crate) struct PropertyAccessExpressionParser {
    pub(crate) left: Spanned<Expression>,
}

impl Parser for PropertyAccessExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let start = self.left.span().start();

        state.next_token();

        Ok(Expression::Property {
            left: Box::new(self.left),
            right: state.consume_identifier("property")?,
        }
        .at(Span::new(start, state.current.span().end(), state.file_id)))
    }
}

pub(crate) struct PrefixExpressionParser;

impl Parser for PrefixExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let operator = state.next.clone();
        state.next_token();

        let inner = ExpressionParser {
            precedence: Precedence::Unary,
        }
        .parse_with(state)?;

        let span = Span::new(operator.span().start(), inner.span().end(), state.file_id);

        Ok(Expression::Unary {
            inner: Box::new(inner),
            operator,
            postfix: false,
        }
        .at(span))
    }
}

pub(crate) struct PostfixExpressionParser {
    pub(crate) left: Spanned<Expression>,
}

impl Parser for PostfixExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let start = self.left.span().start();

        state.next_token();

        Ok(Expression::Unary {
            inner: Box::new(self.left),
            operator: state.current.clone(),
            postfix: true,
        }
        .at(Span::new(start, state.current.span().end(), state.file_id)))
    }
}

pub(crate) struct ParenthesizedExpressionParser;

impl Parser for ParenthesizedExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();
        let start = state.current.span().start();

        let inner = ExpressionParser {
            precedence: Precedence::Lowest,
        }
        .parse_with(state)?;

        state.consume(Token![')'], "parenthesized expression")?;

        Ok(Expression::Parenthesized {
            inner: Box::new(inner),
        }
        .at(Span::new(start, state.current.span().end(), state.file_id)))
    }
}

pub(crate) struct IfExpressionParser;

impl Parser for IfExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();

        let start = state.current.span().start();

        let mut if_blocks = vec![(
            ExpressionParser::default().parse_with(state)?,
            StatementsBlockParser.parse_with(state)?,
        )];

        let mut r#else = None;

        while *state.next.unwrap() == Token![else] {
            state.next_token();

            match state.next.unwrap() {
                Token![if] => {}
                _ => {
                    r#else = Some(StatementsBlockParser.parse_with(state)?);
                    break;
                }
            }

            state.next_token();

            if_blocks.push((
                ExpressionParser::default().parse_with(state)?,
                StatementsBlockParser.parse_with(state)?,
            ));
        }

        Ok(Expression::If { if_blocks, r#else }.at(Span::new(
            start,
            state.current.span().end(),
            state.file_id,
        )))
    }
}

pub(crate) struct CastExpressionParser {
    pub(crate) left: Spanned<Expression>,
}

impl Parser for CastExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let start = self.left.span().start();

        state.next_token();

        let right = TypeParser.parse_with(state)?;

        Ok(Expression::As {
            left: Box::new(self.left),
            right,
        }
        .at(Span::new(start, state.current.span().end(), state.file_id)))
    }
}

pub(crate) struct CallExpressionParser {
    pub(crate) left: Spanned<Expression>,
}

impl Parser for CallExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let start = self.left.span().start();

        state.next_token();

        let arguments = parse_list!(state, "call arguments list", Token![')'], || {
            ExpressionParser {
                precedence: Precedence::Lowest,
            }
            .parse_with(state)
        });

        state.next_token();

        Ok(Expression::Call {
            left: Box::new(self.left),
            arguments,
        }
        .at(Span::new(start, state.current.span().end(), state.file_id)))
    }
}

pub(crate) struct BinaryExpressionParser {
    pub(crate) left: Spanned<Expression>,
}

impl Parser for BinaryExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let start = self.left.span().start();

        let operator = state.next.clone();
        let precedence = state.next.unwrap().to_precedence();

        state.next_token();

        let right = ExpressionParser { precedence }.parse_with(state)?;

        Ok(Expression::Binary {
            left: Box::new(self.left),
            right: Box::new(right),
            operator,
        }
        .at(Span::new(start, state.current.span().end(), state.file_id)))
    }
}

pub(crate) struct ArrayLiteralExpressionParser;

impl Parser for ArrayLiteralExpressionParser {
    type Output = Spanned<Expression>;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();

        let start = state.next.span().start();

        let elements = parse_list!(state, "array literal", Token![']'], || {
            ExpressionParser::default().parse_with(state)
        });

        state.next_token();

        Ok(Expression::Array { elements }.at(Span::new(
            start,
            state.current.span().end(),
            state.file_id,
        )))
    }
}

#[cfg(test)]
mod expression_tests {
    use crate::macros::parser_test;

    parser_test!(ExpressionParser, literal1, "3");
    parser_test!(ExpressionParser, literal2, "\"hello\"");
    parser_test!(ExpressionParser, literal3, "true");
    parser_test!(ExpressionParser, array, "[1, 2, \"3\".into()]");
    parser_test!(
        ExpressionParser,
        binary,
        "!(1 + 2) + 3 / (3 + a.unwrap_or(0) * 4)"
    );
    parser_test!(ExpressionParser, cast, "1 as f32");
    parser_test!(ExpressionParser, call, "l(2 * b() + 2).a()");
    parser_test!(ExpressionParser, call_with_generics, "sizeof[i32]()");
    parser_test!(
        ExpressionParser,
        ifelse,
        "if false { 2.3 } else if false { 5 as f32 } else { 2.0 }"
    );
    parser_test!(
        ExpressionParser,
        r#while,
        "while true { print(\"hello\"); }"
    );
    parser_test!(ExpressionParser, postfix, "Some(a().unwrap_or(0) + b()?)");
}
