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
    span::At,
    token::{Keyword::*, Punctuator::*, RawToken::*},
};

#[derive(Default)]
pub(crate) struct ExpressionParser {
    pub(crate) precedence: Precedence,
}

impl Parser for ExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let mut left = PrimaryExpressionParser.parse_with(state)?;

        while self.precedence < state.next.inner.to_precedence() {
            left = match &state.next.inner {
                binop_pattern!() => BinaryExpressionParser { left }.parse_with(state)?,
                Punctuator(OpenParent) => CallExpressionParser { left }.parse_with(state)?,
                Punctuator(Dot) => PropertyAccessExpressionParser { left }.parse_with(state)?,
                Punctuator(OpenBracket) => {
                    TypeAnnotationsExpressionParser { left }.parse_with(state)?
                }
                postfixop_pattern!() => PostfixExpressionParser { left }.parse_with(state)?,
                Keyword(As) => CastExpressionParser { left }.parse_with(state)?,
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
        state.advance();
        let start = state.current.span.start;

        let condition = ExpressionParser::default().parse_with(state)?;
        let body = StatementsBlockParser.parse_with(state)?;

        Ok(RawExpression::from(WhileExpression {
            condition: Box::new(condition),
            body,
        })
        .at(start..state.current.span.end))
    }
}

pub(crate) struct PrimaryExpressionParser;

impl Parser for PrimaryExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        match state.next.inner.clone() {
            IntegerLiteral(literal) => {
                state.advance();
                Ok(
                    RawExpression::from(IntegerLiteralExpression { literal })
                        .at(state.current.span),
                )
            }
            FloatLiteral(literal) => {
                state.advance();
                Ok(RawExpression::from(FloatLiteralExpression { literal }).at(state.current.span))
            }
            ImaginaryNumberLiteral(literal) => {
                state.advance();
                Ok(
                    RawExpression::from(ImaginaryNumberLiteralExpression { literal })
                        .at(state.current.span),
                )
            }
            StringLiteral(literal) => {
                state.advance();
                Ok(RawExpression::from(StringLiteralExpression { literal }).at(state.current.span))
            }
            CharLiteral(literal) => {
                state.advance();
                Ok(RawExpression::from(CharLiteralExpression { literal }).at(state.current.span))
            }
            BoolLiteral(literal) => {
                state.advance();
                Ok(RawExpression::from(BoolLiteralExpression { literal }).at(state.current.span))
            }
            prefixop_pattern!() => PrefixExpressionParser.parse_with(state),
            Punctuator(OpenParent) => ParenthesizedExpressionParser.parse_with(state),
            Punctuator(OpenBracket) => ArrayLiteralExpressionParser.parse_with(state),
            Identifier(name) => {
                state.advance();
                Ok(RawExpression::from(IdentifierExpression { name }).at(state.current.span))
            }
            Keyword(If) => IfExpressionParser.parse_with(state),
            Keyword(While) => WhileExpressionParser.parse_with(state),
            _ => Err(ParseError::unexpected_token(
                state.next.clone(),
                expected!(
                    "integer literal",
                    "float literal",
                    "imaginary number literal",
                    "string literal",
                    "char literal",
                    "boolean literal",
                    Punctuator(OpenParent),
                    Punctuator(OpenBracket),
                    "identifier",
                    Keyword(If),
                    Keyword(While)
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
    parser_test!(ExpressionParser, binary, "!(1 + 2) + 3 / (3 + a ?: 0 * 4)");
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
    parser_test!(ExpressionParser, postfix, "Some(a() ?: 0 + b()?)");
}
