use crate::{error::ParserError, macros::*, Parser, ParserResult};
use num_traits::ToPrimitive;
use ry_ast::{
    expression::Expression, precedence::Precedence, span::WithSpannable, token::RawToken::*, *,
};

impl<'c> Parser<'c> {
    pub(crate) fn parse_expression(&mut self, precedence: i8) -> ParserResult<Expression> {
        let mut left = self.parse_prefix()?;

        while precedence < self.next.value().to_precedence() {
            left = match &self.next.value() {
                binop_pattern!() => {
                    let op = self.next.clone();
                    let precedence = self.next.value().to_precedence();
                    self.advance()?; // op

                    let right = self.parse_expression(precedence)?;

                    Box::new(RawExpression::Binary(left, op, right))
                        .with_span(left.span().start()..self.current.span().end())
                }
                OpenParent => {
                    self.advance()?; // `(`

                    let arguments =
                        parse_list!(self, "call arguments list", CloseParent, false, || self
                            .parse_expression(Precedence::Lowest.to_i8().unwrap()));

                    self.advance()?; // `)`

                    Box::new(RawExpression::Call(vec![], left, arguments))
                        .with_span(left.span().start()..self.current.span().end())
                }
                Dot => {
                    self.advance()?; // `.`

                    let name = consume_ident!(self, "property");

                    Box::new(RawExpression::Property(left, name))
                        .with_span(left.span().start()..name.span().end())
                }
                OpenBracket => {
                    self.advance()?; // `[`

                    let generics = parse_list!(self, "list literal", CloseBracket, false, || {
                        self.parse_type()
                    });

                    let end = self.current.span().end();

                    self.advance()?;

                    Box::new(RawExpression::Generics(left, generics))
                        .with_span(left.span().start()..end)
                }
                postfixop_pattern!() => {
                    self.advance()?; // `?`

                    let right = self.current.clone();

                    Box::new(RawExpression::PrefixOrPostfix(false, right, left))
                        .with_span(left.start().span()..right.span().end())
                }
                As => {
                    self.advance()?; // `as`

                    let r#type = self.parse_type()?;

                    Box::new(RawExpression::As(left, r#type))
                        .with_span(left.span().start()..self.current.span().end())
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

                Ok(Box::new(RawExpression::Int(value)).with_span(self.current.span()))
            }
            Float(f) => {
                let value = *f;

                self.advance()?; // float

                Ok(Box::new(RawExpression::Float(value)).with_span(self.current.span()))
            }
            Imag(i) => {
                let value = *i;

                self.advance()?; // imag

                Ok(Box::new(RawExpression::Imag(value)).with_span(self.current.span()))
            }
            String(s) => {
                let value = s.clone();

                self.advance()?; // string

                Ok(Box::new(RawExpression::String(value)).with_span(self.current.span()))
            }
            Char(c) => {
                let value = *c;

                self.advance()?; // char

                Ok(Box::new(RawExpression::Char(value)).with_span(self.current.span()))
            }
            Bool(b) => {
                let value = *b;

                self.advance()?; // bool

                Ok(Box::new(RawExpression::Bool(value)).with_span(self.current.span()))
            }
            prefixop_pattern!() => {
                let left = self.next.clone();
                self.advance()?; // left

                let expr = self.parse_expression(Precedence::PrefixOrPostfix.to_i8().unwrap())?;

                Ok(Box::new(RawExpression::PrefixOrPostfix(true, left, expr)).with_span(
                    left.span().start()..expr.span().end()))
            }
            OpenParent => {
                self.advance()?; // `(`

                let expr = self.parse_expression(Precedence::Lowest.to_i8().unwrap())?;

                consume!(self, CloseParent, "parenthesized expression");

                Ok(expr)
            }
            OpenBracket => {
                self.advance()?; // `[`

                let start = self.next.span().start();

                let list = parse_list!(self, "list literal", CloseBracket, false, || {
                    self.parse_expression(Precedence::Lowest.to_i8().unwrap())
                });

                let end = self.current.span().end();

                Ok(Box::new(RawExpression::List(list)).with_span(start..end))
            }
            Identifier(n) => {
                let result = Box::new(
                    RawExpression::Name(*n))
                    .with_span(self.current.span());

                self.advance()?;

                Ok(result)
            }
            If => {
                self.advance()?;
                let start = self.current.span().start();

                let if_condition = self.parse_expression(Precedence::Lowest.to_i8().unwrap())?;
                let if_statements_block = self.parse_statements_block(false)?;

                let mut else_statements_block = None;
                let mut else_if_chains = vec![];

                while self.next.unwrap().is(Else) {
                    self.advance()?; // `else`

                    if !self.next.unwrap().is(If) {
                        else_statements_block = Some(self.parse_statements_block(false)?);
                        break;
                    }

                    self.advance()?; // `if`

                    let else_if_condition =
                        self.parse_expression(Precedence::Lowest.to_i8().unwrap())?;

                    let else_if_statements_block = self.parse_statements_block(false)?;

                    else_if_chains.push((else_if_condition, else_if_statements_block));
                }

                let end = self.current.span().end();

                Ok(Box::new(RawExpression::If(
                    (if_condition, if_statements_block),
                    else_if_chains,
                    else_statements_block,
                ))
                .with_span(start..end))
            }
            While => {
                self.advance()?;
                let start = self.current.span().start();

                let condition = self.parse_expression(Precedence::Lowest.to_i8().unwrap())?;
                let block = self.parse_statements_block(false)?;

                Ok(Box::new(RawExpression::While(condition, block)).with_span(start..self.current.span().end()))
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

    parser_test!(literal1, "fun test() i32 { 3 }");
    parser_test!(literal2, "fun test() string { \"hello\" }");
    parser_test!(literal3, "fun test() bool { true }");
    parser_test!(binary1, "fun test() i32 { 2 + 3 }");
    parser_test!(binary2, "fun test() f32 { 1 + 2 / 3 + 3 * 4 }");
    parser_test!(r#as, "fun test() f32 { 1 as f32 }");
    parser_test!(call, "fun test() f32 { l(2 + 3).a() }");
    parser_test!(call_with_generics, "fun test() f32 { l[i32](2 + 3).a[]() }");
    parser_test!(
        ifelse,
        "fun test() f32 { if false { 2.3 } else if false { 5 as f32 } else { 2.0 } }"
    );
    parser_test!(r#while, "fun test() { while true { print(\"hello\"); } }");
    parser_test!(postfix, "fun test() i32? { Some(a() ?: 0 + b()?) }");
    parser_test!(parent, "fun test() i32 { ((b + c) * d) }");
}
