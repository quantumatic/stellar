use crate::{error::*, macros::*, Parser};
use ry_ast::{
    expression::*,
    precedence::Precedence,
    span::At,
    token::{Keyword::*, Punctuator::*, RawToken::*},
};

impl Parser<'_> {
    pub(crate) fn parse_expression(&mut self, precedence: Precedence) -> ParseResult<Expression> {
        let mut left = self.parse_prefix()?;

        while precedence < self.next.inner.to_precedence() {
            left = match &self.next.inner {
                binop_pattern!() => {
                    let op = self.next.clone();
                    let precedence = self.next.inner.to_precedence();

                    self.advance();

                    let right = self.parse_expression(precedence)?;

                    let span = left.span.start()..self.current.span.end();

                    RawExpression::from(BinaryExpression {
                        left: Box::new(left),
                        right: Box::new(right),
                        op,
                    })
                    .at(span)
                }
                Punctuator(OpenParent) => {
                    self.advance();

                    let arguments = parse_list!(
                        self,
                        "call arguments list",
                        Punctuator(CloseParent),
                        false,
                        || self.parse_expression(Precedence::Lowest)
                    );

                    self.advance();

                    let span = left.span.start()..self.current.span.end();

                    RawExpression::from(CallExpression {
                        left: Box::new(left),
                        arguments,
                    })
                    .at(span)
                }
                Punctuator(Dot) => {
                    self.advance();

                    let property = self.consume_identifier("property")?;

                    let span = left.span.start()..property.span.end();

                    RawExpression::from(PropertyAccessExpression {
                        left: Box::new(left),
                        property,
                    })
                    .at(span)
                }
                Punctuator(OpenBracket) => {
                    self.advance();

                    let type_annotations = parse_list!(
                        self,
                        "type annotations",
                        Punctuator(CloseBracket),
                        false,
                        || { self.parse_type() }
                    );

                    let span = left.span.start()..self.current.span.end();

                    self.advance();

                    RawExpression::from(TypeAnnotationsExpression {
                        left: Box::new(left),
                        type_annotations,
                    })
                    .at(span)
                }
                postfixop_pattern!() => {
                    self.advance();

                    let op = self.current.clone();

                    let span = left.span.start()..op.span.end();

                    RawExpression::from(UnaryExpression {
                        inner: Box::new(left),
                        op,
                        postfix: true,
                    })
                    .at(span)
                }
                Keyword(As) => {
                    self.advance();

                    let right = self.parse_type()?;

                    let span = left.span.start()..self.current.span.end();

                    RawExpression::from(AsExpression {
                        left: Box::new(left),
                        right,
                    })
                    .at(span)
                }
                _ => break,
            };
        }

        Ok(left)
    }

    pub(crate) fn parse_prefix(&mut self) -> ParseResult<Expression> {
        match &self.next.inner {
            IntegerLiteral(literal) => {
                let literal = *literal;
                self.advance();
                Ok(RawExpression::from(IntegerLiteralExpression { literal }).at(self.current.span))
            }
            FloatLiteral(literal) => {
                let literal = *literal;
                self.advance();
                Ok(RawExpression::from(FloatLiteralExpression { literal }).at(self.current.span))
            }
            ImaginaryNumberLiteral(literal) => {
                let literal = *literal;
                self.advance();
                Ok(RawExpression::from(ImaginaryNumberLiteralExpression { literal }).at(self.current.span))
            }
            StringLiteral(literal) => {
                let literal = literal.clone();
                self.advance();
                Ok(RawExpression::from(StringLiteralExpression { literal }).at(self.current.span))
            }
            CharLiteral(literal) => {
                let literal = *literal;
                self.advance();
                Ok(RawExpression::from(CharLiteralExpression { literal }).at(self.current.span))
            }
            BoolLiteral(literal) => {
                let literal = *literal;
                self.advance();
                Ok(RawExpression::from(BoolLiteralExpression { literal }).at(self.current.span))
            }
            prefixop_pattern!() => {
                let op = self.next.clone();
                self.advance();

                let inner = self.parse_expression(Precedence::Unary)?;
                let span = op.span.start()..inner.span.end();

                Ok(RawExpression::from(UnaryExpression { inner: Box::new(inner), op, postfix: false }).at(span))
            }
            Punctuator(OpenParent) => {
                self.advance();

                let expression = self.parse_expression(Precedence::Lowest)?;

                self.consume(Punctuator(CloseParent), "parenthesized expression")?;

                Ok(expression)
            }
            Punctuator(OpenBracket) => {
                self.advance();

                let start = self.next.span.start();

                let literal = parse_list!(self, "array literal", Punctuator(CloseBracket), false, || {
                    self.parse_expression(Precedence::Lowest)
                });

                let end = self.current.span.end();

                Ok(RawExpression::from(ArrayLiteralExpression { literal }).at(start..end))
            }
            Identifier(name) => {
                let result =
                RawExpression::from(IdentifierExpression { name: *name })
                    .at(self.current.span);

                self.advance();

                Ok(result)
            }
            Keyword(If) => {
                self.advance();

                let start = self.current.span.start();

                let mut if_blocks = vec![IfBlock {
                    condition: self.parse_expression(Precedence::Lowest)?,
                    body: self.parse_statements_block(false)?
                }];

                let mut r#else = None;

                while let Keyword(Else) = self.next.inner {
                    self.advance();

                    match self.next.inner {
                        Keyword(If) => {},
                        _ => {
                            r#else = Some(self.parse_statements_block(false)?);
                            break;
                        }
                    }

                    self.advance();

                    if_blocks.push(IfBlock {
                        condition: self.parse_expression(Precedence::Lowest)?,
                        body: self.parse_statements_block(false)?
                    });
                }

                let end = self.current.span.end();

                Ok(RawExpression::from(IfExpression { if_blocks, r#else })
                    .at(start..end))
            }
            Keyword(While) => {
                self.advance();
                let start = self.current.span.start();

                let condition = self.parse_expression(Precedence::Lowest)?;
                let body = self.parse_statements_block(false)?;

                Ok(RawExpression::from(WhileExpression { condition: Box::new(condition), body })
                    .at(start..self.current.span.end()))
            }
            _ => Err(ParseError::unexpected_token(
                self.next.clone(),
                "integer, float, imaginary, string literals, ... or identifier for name, `(`, `[`, `if`, `while`, ...",
                "expression"
            )),
        }
    }
}

#[cfg(test)]
mod expression_tests {
    use crate::{macros::parser_test, Parser};
    use ry_interner::Interner;

    parser_test!(literal1, "fun test(): i32 { 3 }");
    parser_test!(literal2, "fun test(): String { \"hello\" }");
    parser_test!(literal3, "fun test(): bool { true }");
    parser_test!(binary1, "fun test(): i32 { 2 + 3 }");
    parser_test!(binary2, "fun test(): f32 { 1 + 2 / 3 + 3 * 4 }");
    parser_test!(r#as, "fun test(): f32 { 1 as f32 }");
    parser_test!(call, "fun test(): f32 { l(2 + 3).a() }");
    parser_test!(
        call_with_generics,
        "fun test(): f32 { l[i32](2 + 3).a[]() }"
    );
    parser_test!(
        ifelse,
        "fun test(): f32 { if false { 2.3 } else if false { 5 as f32 } else { 2.0 } }"
    );
    parser_test!(r#while, "fun test() { while true { print(\"hello\"); } }");
    parser_test!(postfix, "fun test(): Option[i32] { Some(a() ?: 0 + b()?) }");
    parser_test!(parent, "fun test(): i32 { ((b + c) * d) }");
}
