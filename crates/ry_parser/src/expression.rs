use crate::{error::ParserError, macros::*, Parser, ParserResult};
use num_traits::ToPrimitive;
use ry_ast::{expression::*, precedence::Precedence, span::WithSpan, token::RawToken::*};

impl<'c> Parser<'c> {
    pub(crate) fn parse_expression(&mut self, precedence: i8) -> ParserResult<Expression> {
        let mut left = self.parse_prefix()?;

        while precedence < self.next.unwrap().to_precedence() {
            left = match &self.next.unwrap() {
                binop_pattern!() => {
                    let op = self.next.clone();
                    let precedence = self.next.unwrap().to_precedence();

                    self.advance()?; // op

                    let right = self.parse_expression(precedence)?;

                    let span = left.span().start()..self.current.span().end();

                    RawExpression::from(BinaryExpression::new(left, right, op)).with_span(span)
                }
                OpenParent => {
                    self.advance()?; // `(`

                    let arguments =
                        parse_list!(self, "call arguments list", CloseParent, false, || self
                            .parse_expression(Precedence::Lowest.to_i8().unwrap()));

                    self.advance()?; // `)`

                    let span = left.span().start()..self.current.span().end();

                    RawExpression::from(CallExpression::new(left, arguments)).with_span(span)
                }
                Dot => {
                    self.advance()?; // `.`

                    let name = consume_ident!(self, "property");

                    let span = left.span().start()..name.span().end();

                    RawExpression::from(PropertyAccessExpression::new(left, name)).with_span(span)
                }
                OpenBracket => {
                    self.advance()?; // `[`

                    let type_annotations =
                        parse_list!(self, "type annotations", CloseBracket, false, || {
                            self.parse_type()
                        });

                    let span = left.span().start()..self.current.span().end();

                    self.advance()?;

                    RawExpression::from(TypeAnnotationsExpression::new(left, type_annotations))
                        .with_span(span)
                }
                postfixop_pattern!() => {
                    self.advance()?; // `?`

                    let right = self.current.clone();

                    let span = left.span().start()..right.span().end();

                    RawExpression::from(UnaryExpression::new(left, right, true)).with_span(span)
                }
                As => {
                    self.advance()?; // `as`

                    let r#type = self.parse_type()?;

                    let span = left.span().start()..self.current.span().end();

                    RawExpression::from(AsExpression::new(left, r#type)).with_span(span)
                }
                _ => break,
            };
        }

        Ok(left)
    }

    pub(crate) fn parse_prefix(&mut self) -> ParserResult<Expression> {
        self.check_scanning_error_for_next_token()?;

        match self.next.unwrap() {
            Int(i) => {
                let value = *i;

                self.advance()?; // int

                Ok(RawExpression::from(IntegerLiteralExpression::new(value)).with_span(self.current.span()))
            }
            Float(f) => {
                let value = *f;

                self.advance()?; // float

                Ok(RawExpression::from(FloatLiteralExpression::new(value)).with_span(self.current.span()))
            }
            Imag(i) => {
                let value = *i;

                self.advance()?; // imag

                Ok(RawExpression::from(ImaginaryNumberLiteralExpression::new(value)).with_span(self.current.span()))
            }
            String(s) => {
                let value = s.clone();

                self.advance()?; // string

                Ok(RawExpression::from(StringLiteralExpression::new(value)).with_span(self.current.span()))
            }
            Char(c) => {
                let value = *c;

                self.advance()?; // char

                Ok(RawExpression::from(CharLiteralExpression::new(value)).with_span(self.current.span()))
            }
            Bool(b) => {
                let value = *b;

                self.advance()?; // bool

                Ok(RawExpression::from(BoolLiteralExpression::new(value)).with_span(self.current.span()))
            }
            prefixop_pattern!() => {
                let left = self.next.clone();
                self.advance()?; // left

                let right = self.parse_expression(Precedence::Unary.to_i8().unwrap())?;
                let span = left.span().start()..right.span().end();

                Ok(RawExpression::from(UnaryExpression::new(right, left, false)).with_span(span))
            }
            OpenParent => {
                self.advance()?; // `(`

                let expression = self.parse_expression(Precedence::Lowest.to_i8().unwrap())?;

                consume!(self, CloseParent, "parenthesized expression");

                Ok(expression)
            }
            OpenBracket => {
                self.advance()?; // `[`

                let start = self.next.span().start();

                let array = parse_list!(self, "array literal", CloseBracket, false, || {
                    self.parse_expression(Precedence::Lowest.to_i8().unwrap())
                });

                let end = self.current.span().end();

                Ok(RawExpression::from(ArrayLiteralExpression::new(array)).with_span(start..end))
            }
            Identifier(n) => {
                let result =
                RawExpression::from(IdentifierExpression::new(*n))
                    .with_span(self.current.span());

                self.advance()?;

                Ok(result)
            }
            If => {
                self.advance()?; // `if`

                let start = self.current.span().start();

                let mut if_blocks = vec![IfBlock::new(
                    self.parse_expression(Precedence::Lowest.to_i8().unwrap())?,
                    self.parse_statements_block(false)?
                )];

                let mut r#else = None;

                while self.next.unwrap().is(Else) {
                    self.advance()?; // `else`

                    if !self.next.unwrap().is(If) {
                        r#else = Some(self.parse_statements_block(false)?);
                        break;
                    }

                    self.advance()?; // `if`

                    if_blocks.push(IfBlock::new(
                        self.parse_expression(Precedence::Lowest.to_i8().unwrap())?,
                        self.parse_statements_block(false)?
                    ));
                }

                let end = self.current.span().end();

                Ok(RawExpression::from(IfExpression::new(if_blocks, r#else))
                    .with_span(start..end))
            }
            While => {
                self.advance()?;
                let start = self.current.span().start();

                let condition = self.parse_expression(Precedence::Lowest.to_i8().unwrap())?;
                let block = self.parse_statements_block(false)?;

                Ok(RawExpression::from(WhileExpression::new(condition, block))
                    .with_span(start..self.current.span().end()))
            }
            _ => Err(ParserError::UnexpectedToken(
                self.next.clone(),
                "integer, float, imaginary, string literals, ... or identifier for name, `(`, `[`, `if`, `while`, ..."
                    .to_owned(),
                "expression".to_owned(),
            )),
        }
    }
}

#[cfg(test)]
mod expression_tests {
    use crate::{macros::parser_test, Parser};
    use string_interner::StringInterner;

    parser_test!(literal1, "fun test(): i32 { 3 }");
    parser_test!(literal2, "fun test(): string { \"hello\" }");
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
    parser_test!(postfix, "fun test(): i32? { Some(a() ?: 0 + b()?) }");
    parser_test!(parent, "fun test(): i32 { ((b + c) * d) }");
}
