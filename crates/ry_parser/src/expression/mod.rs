pub(crate) mod array;
pub(crate) mod binary;
pub(crate) mod call;
pub(crate) mod cast;
pub(crate) mod r#if;
pub(crate) mod parenthesized;
pub(crate) mod postfix;
pub(crate) mod prefix;
pub(crate) mod property;
pub(crate) mod type_annotations;
pub(crate) mod r#while;

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

pub(crate) struct ExpressionParser {
    pub(crate) precedence: Precedence,
}

impl Default for ExpressionParser {
    fn default() -> Self {
        Self {
            precedence: Precedence::Lowest,
        }
    }
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

// #[cfg(test)]
// mod expression_tests {
//     use crate::{macros::parser_test, Parser};
//     use ry_interner::Interner;

//     parser_test!(literal1, "fun test(): i32 { 3 }");
//     parser_test!(literal2, "fun test(): String { \"hello\" }");
//     parser_test!(literal3, "fun test(): bool { true }");
//     parser_test!(binary1, "fun test(): i32 { 2 + 3 }");
//     parser_test!(binary2, "fun test(): f32 { 1 + 2 / 3 + 3 * 4 }");
//     parser_test!(r#as, "fun test(): f32 { 1 as f32 }");
//     parser_test!(call, "fun test(): f32 { l(2 + 3).a() }");
//     parser_test!(
//         call_with_generics,
//         "fun test(): f32 { l[i32](2 + 3).a[]() }"
//     );
//     parser_test!(
//         ifelse,
//         "fun test(): f32 { if false { 2.3 } else if false { 5 as f32 } else { 2.0 } }"
//     );
//     parser_test!(r#while, "fun test() { while true { print(\"hello\"); } }");
//     parser_test!(postfix, "fun test(): Option[i32] { Some(a() ?: 0 + b()?) }");
//     parser_test!(parent, "fun test(): i32 { ((b + c) * d) }");
// }
