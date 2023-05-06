mod array;
mod binary;
mod call;
mod cast;
mod r#if;
mod parenthesized;
mod postfix;
mod prefix;
mod property;
mod type_annotations;
mod r#while;

use self::{
    array::ArrayLiteralExpressionParser, binary::BinaryExpressionParser,
    call::CallExpressionParser, cast::CastExpressionParser,
    parenthesized::ParenthesizedExpressionParser, postfix::PostfixExpressionParser,
    prefix::PrefixExpressionParser, property::PropertyAccessExpressionParser,
    r#if::IfExpressionParser, type_annotations::TypeAnnotationsExpressionParser,
};
use crate::{
    error::{expected, ParseError, ParseResult},
    macros::{binop_pattern, postfixop_pattern, prefixop_pattern},
    statement::StatementsBlockParser,
    Parser, ParserState,
};
use ry_ast::{
    expression::*,
    precedence::Precedence,
    span::{At, Span, SpanIndex},
    token::RawToken,
    Token,
};

#[derive(Default)]
pub(crate) struct ExpressionParser {
    pub(crate) precedence: Precedence,
}

impl Parser for ExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let mut left = PrimaryExpressionParser.parse_with(state)?;

        while self.precedence < state.next.unwrap().to_precedence() {
            left = match state.next.unwrap() {
                binop_pattern!() => BinaryExpressionParser { left }.parse_with(state)?,
                Token!['('] => CallExpressionParser { left }.parse_with(state)?,
                Token![.] => PropertyAccessExpressionParser { left }.parse_with(state)?,
                Token!['['] => TypeAnnotationsExpressionParser { left }.parse_with(state)?,
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
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.next_token();
        let start = state.current.span().start();

        let condition = ExpressionParser::default().parse_with(state)?;
        let body = StatementsBlockParser.parse_with(state)?;

        Ok(RawExpression::from(WhileExpression {
            condition: Box::new(condition),
            body,
        })
        .at(Span::new(start, state.current.span().end(), state.file_id)))
    }
}

pub(crate) struct PrimaryExpressionParser;

impl Parser for PrimaryExpressionParser {
    type Output = Expression;

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
                    Ok(literal) => Ok(RawExpression::from(IntegerLiteralExpression { literal })
                        .at(state.current.span())),
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
                    Ok(literal) => Ok(RawExpression::from(FloatLiteralExpression { literal })
                        .at(state.current.span())),
                    Err(..) => Err(ParseError::float_overflow(state.current.span())),
                }
            }
            RawToken::StringLiteral => {
                state.next_token();
                Ok(RawExpression::from(StringLiteralExpression {
                    literal: state.contents.index(state.current.span()).to_owned(),
                })
                .at(state.current.span()))
            }
            RawToken::CharLiteral => {
                state.next_token();
                Ok(RawExpression::from(StringLiteralExpression {
                    literal: state.contents.index(state.current.span()).to_owned(),
                })
                .at(state.current.span()))
            }
            Token![true] => {
                state.next_token();
                Ok(RawExpression::from(BoolLiteralExpression { literal: true })
                    .at(state.current.span()))
            }
            Token![false] => {
                state.next_token();
                Ok(
                    RawExpression::from(BoolLiteralExpression { literal: false })
                        .at(state.current.span()),
                )
            }
            prefixop_pattern!() => PrefixExpressionParser.parse_with(state),
            Token!['('] => ParenthesizedExpressionParser.parse_with(state),
            Token!['['] => ArrayLiteralExpressionParser.parse_with(state),
            RawToken::Identifier(name) => {
                state.next_token();
                Ok(RawExpression::from(IdentifierExpression { name }).at(state.current.span()))
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
