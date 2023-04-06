use crate::{
    error::*, macros::*, r#type::TypeParser, statement::StatementsBlockParser, Parser, ParserState,
};
use ry_ast::{
    expression::*,
    precedence::Precedence,
    span::At,
    token::{Keyword::*, Punctuator::*, RawToken::*},
};

pub(crate) struct BinaryExpressionParser {
    pub(crate) left: Expression,
}

impl Parser for BinaryExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let op = state.next.clone();
        let precedence = state.next.inner.to_precedence();

        state.advance();

        let right = ExpressionParser { precedence }.parse_with(state)?;

        let span = self.left.span.start..state.current.span.end;

        Ok(RawExpression::from(BinaryExpression {
            left: Box::new(self.left),
            right: Box::new(right),
            op,
        })
        .at(span))
    }
}

pub(crate) struct CallExpressionParser {
    pub(crate) left: Expression,
}

impl Parser for CallExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.advance();

        let arguments = parse_list!(
            state,
            "call arguments list",
            Punctuator(CloseParent),
            || ExpressionParser {
                precedence: Precedence::Lowest
            }
            .parse_with(state)
        );

        state.advance();

        let span = self.left.span.start..state.current.span.end;

        Ok(RawExpression::from(CallExpression {
            left: Box::new(self.left),
            arguments,
        })
        .at(span))
    }
}

pub(crate) struct PropertyAccessExpressionParser {
    pub(crate) left: Expression,
}

impl Parser for PropertyAccessExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.advance();

        let property = state.consume_identifier("property")?;

        let span = self.left.span.start..property.span.end;

        Ok(RawExpression::from(PropertyAccessExpression {
            left: Box::new(self.left),
            property,
        })
        .at(span))
    }
}

pub(crate) struct TypeAnnotationsExpressionParser {
    pub(crate) left: Expression,
}

impl Parser for TypeAnnotationsExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.advance();

        let type_annotations =
            parse_list!(state, "type annotations", Punctuator(CloseBracket), || {
                TypeParser.parse_with(state)
            });

        let span = self.left.span.start..state.current.span.end;

        state.advance();

        Ok(RawExpression::from(TypeAnnotationsExpression {
            left: Box::new(self.left),
            type_annotations,
        })
        .at(span))
    }
}

pub(crate) struct PostfixExpressionParser {
    pub(crate) left: Expression,
}

impl Parser for PostfixExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.advance();

        let op = state.current.clone();

        let span = self.left.span.start..op.span.end;

        Ok(RawExpression::from(UnaryExpression {
            inner: Box::new(self.left),
            op,
            postfix: true,
        })
        .at(span))
    }
}

pub(crate) struct CastExpressionParser {
    pub(crate) left: Expression,
}

impl Parser for CastExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.advance();

        let right = TypeParser.parse_with(state)?;

        let span = self.left.span.start..state.current.span.end;

        Ok(RawExpression::from(AsExpression {
            left: Box::new(self.left),
            right,
        })
        .at(span))
    }
}

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

pub(crate) struct PrefixExpressionParser;

impl Parser for PrefixExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        let op = state.next.clone();
        state.advance();

        let inner = ExpressionParser {
            precedence: Precedence::Unary,
        }
        .parse_with(state)?;
        let span = op.span.start..inner.span.end;

        Ok(RawExpression::from(UnaryExpression {
            inner: Box::new(inner),
            op,
            postfix: false,
        })
        .at(span))
    }
}

pub(crate) struct ParenthesizedExpressionParser;

impl Parser for ParenthesizedExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.advance();

        let expression = ExpressionParser {
            precedence: Precedence::Lowest,
        }
        .parse_with(state)?;

        state.consume(Punctuator(CloseParent), "parenthesized expression")?;

        Ok(expression)
    }
}

pub(crate) struct ArrayLiteralExpressionParser;

impl Parser for ArrayLiteralExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.advance();

        let start = state.next.span.start;

        let literal = parse_list!(state, "array literal", Punctuator(CloseBracket), || {
            ExpressionParser::default().parse_with(state)
        });

        let end = state.current.span.end;

        Ok(RawExpression::from(ArrayLiteralExpression { literal }).at(start..end))
    }
}

pub(crate) struct IfExpressionParser;

impl Parser for IfExpressionParser {
    type Output = Expression;

    fn parse_with(self, state: &mut ParserState<'_>) -> ParseResult<Self::Output> {
        state.advance();

        let start = state.current.span.start;

        let mut if_blocks = vec![IfBlock {
            condition: ExpressionParser::default().parse_with(state)?,
            body: StatementsBlockParser.parse_with(state)?,
        }];

        let mut r#else = None;

        while state.next.inner == Keyword(Else) {
            state.advance();

            match state.next.inner {
                Keyword(If) => {}
                _ => {
                    r#else = Some(StatementsBlockParser.parse_with(state)?);
                    break;
                }
            }

            state.advance();

            if_blocks.push(IfBlock {
                condition: ExpressionParser::default().parse_with(state)?,
                body: StatementsBlockParser.parse_with(state)?,
            });
        }

        let end = state.current.span.end;

        Ok(RawExpression::from(IfExpression { if_blocks, r#else }).at(start..end))
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
