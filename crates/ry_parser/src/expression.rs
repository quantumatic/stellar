use crate::{error::*, macros::*, Parser};
use ry_ast::{
    expression::*,
    precedence::Precedence,
    span::WithSpan,
    token::{Keyword::*, Punctuator::*, RawToken::*},
};

impl Parser<'_> {
    pub(crate) fn parse_expression(&mut self, precedence: Precedence) -> ParseResult<Expression> {
        let mut left = self.parse_prefix()?;

        while precedence < self.next.unwrap().to_precedence() {
            left = match &self.next.unwrap() {
                binop_pattern!() => {
                    let op = self.next.clone();
                    let precedence = self.next.unwrap().to_precedence();

                    self.advance()?; // op

                    let right = self.parse_expression(precedence)?;

                    let span = left.span().start()..self.current.span().end();

                    RawExpression::from(BinaryExpression {
                        left: Box::new(left),
                        right: Box::new(right),
                        op,
                    })
                    .with_span(span)
                }
                Punctuator(OpenParent) => {
                    self.advance()?; // `(`

                    let arguments = parse_list!(
                        self,
                        "call arguments list",
                        Punctuator(CloseParent),
                        false,
                        || self.parse_expression(Precedence::Lowest)
                    );

                    self.advance()?; // `)`

                    let span = left.span().start()..self.current.span().end();

                    RawExpression::from(CallExpression {
                        left: Box::new(left),
                        arguments,
                    })
                    .with_span(span)
                }
                Punctuator(Dot) => {
                    self.advance()?; // `.`

                    let property = consume_ident!(self, "property");

                    let span = left.span().start()..property.span().end();

                    RawExpression::from(PropertyAccessExpression {
                        left: Box::new(left),
                        property,
                    })
                    .with_span(span)
                }
                Punctuator(OpenBracket) => {
                    self.advance()?; // `[`

                    let type_annotations = parse_list!(
                        self,
                        "type annotations",
                        Punctuator(CloseBracket),
                        false,
                        || { self.parse_type() }
                    );

                    let span = left.span().start()..self.current.span().end();

                    self.advance()?;

                    RawExpression::from(TypeAnnotationsExpression {
                        left: Box::new(left),
                        type_annotations,
                    })
                    .with_span(span)
                }
                postfixop_pattern!() => {
                    self.advance()?; // `?`

                    let op = self.current.clone();

                    let span = left.span().start()..op.span().end();

                    RawExpression::from(UnaryExpression {
                        inner: Box::new(left),
                        op,
                        postfix: true,
                    })
                    .with_span(span)
                }
                Keyword(As) => {
                    self.advance()?;

                    let right = self.parse_type()?;

                    let span = left.span().start()..self.current.span().end();

                    RawExpression::from(AsExpression {
                        left: Box::new(left),
                        right,
                    })
                    .with_span(span)
                }
                _ => break,
            };
        }

        Ok(left)
    }

    pub(crate) fn parse_prefix(&mut self) -> ParseResult<Expression> {
        match self.next.unwrap() {
            IntegerLiteral(i) => {
                let literal = *i;

                self.advance()?; // int

                Ok(RawExpression::from(IntegerLiteralExpression { literal }).with_span(self.current.span()))
            }
            FloatLiteral(f) => {
                let literal = *f;

                self.advance()?; // float

                Ok(RawExpression::from(FloatLiteralExpression { literal }).with_span(self.current.span()))
            }
            ImaginaryNumberLiteral(i) => {
                let literal = *i;

                self.advance()?; // imag

                Ok(RawExpression::from(ImaginaryNumberLiteralExpression { literal }).with_span(self.current.span()))
            }
            StringLiteral(s) => {
                let literal = s.clone();

                self.advance()?; // string

                Ok(RawExpression::from(StringLiteralExpression { literal }).with_span(self.current.span()))
            }
            CharLiteral(c) => {
                let literal = *c;

                self.advance()?; // char

                Ok(RawExpression::from(CharLiteralExpression { literal }).with_span(self.current.span()))
            }
            BoolLiteral(b) => {
                let literal = *b;

                self.advance()?; // bool

                Ok(RawExpression::from(BoolLiteralExpression { literal }).with_span(self.current.span()))
            }
            prefixop_pattern!() => {
                let op = self.next.clone();
                self.advance()?;

                let inner = self.parse_expression(Precedence::Unary)?;
                let span = op.span().start()..inner.span().end();

                Ok(RawExpression::from(UnaryExpression { inner: Box::new(inner), op, postfix: false }).with_span(span))
            }
            Punctuator(OpenParent) => {
                self.advance()?; // `(`

                let expression = self.parse_expression(Precedence::Lowest)?;

                consume!(self, Punctuator(CloseParent), "parenthesized expression");

                Ok(expression)
            }
            Punctuator(OpenBracket) => {
                self.advance()?; // `[`

                let start = self.next.span().start();

                let literal = parse_list!(self, "array literal", Punctuator(CloseBracket), false, || {
                    self.parse_expression(Precedence::Lowest)
                });

                let end = self.current.span().end();

                Ok(RawExpression::from(ArrayLiteralExpression { literal }).with_span(start..end))
            }
            Identifier(name) => {
                let result =
                RawExpression::from(IdentifierExpression { name: *name })
                    .with_span(self.current.span());

                self.advance()?;

                Ok(result)
            }
            Keyword(If) => {
                self.advance()?; // `if`

                let start = self.current.span().start();

                let mut if_blocks = vec![IfBlock {
                    condition: self.parse_expression(Precedence::Lowest)?,
                    body: self.parse_statements_block(false)?
                }];

                let mut r#else = None;

                while let Keyword(Else) = self.next.unwrap() {
                    self.advance()?; // `else`

                    match self.next.unwrap() {
                        Keyword(If) => {},
                        _ => {
                            r#else = Some(self.parse_statements_block(false)?);
                            break;
                        }
                    }

                    self.advance()?; // `if`

                    if_blocks.push(IfBlock {
                        condition: self.parse_expression(Precedence::Lowest)?,
                        body: self.parse_statements_block(false)?
                    });
                }

                let end = self.current.span().end();

                Ok(RawExpression::from(IfExpression { if_blocks, r#else })
                    .with_span(start..end))
            }
            Keyword(While) => {
                self.advance()?;
                let start = self.current.span().start();

                let condition = self.parse_expression(Precedence::Lowest)?;
                let body = self.parse_statements_block(false)?;

                Ok(RawExpression::from(WhileExpression { condition: Box::new(condition), body })
                    .with_span(start..self.current.span().end()))
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
