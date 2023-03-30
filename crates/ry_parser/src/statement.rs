use crate::{error::ParserError, macros::*, Parser, ParserResult};
use num_traits::ToPrimitive;
use ry_ast::{precedence::Precedence, span::WithSpannable, token::RawToken::*, *};
use std::ops::Deref;

impl<'c> Parser<'c> {
    pub(crate) fn parse_statements_block(
        &mut self,
        top_level: bool,
    ) -> ParserResult<StatementsBlock> {
        consume!(self, OpenBrace, "statements block"); // `{`

        let mut stmts = vec![];

        while !self.next.unwrap().is(CloseBrace) {
            let (stmt, last) = self.parse_statement()?;

            stmts.push(stmt);

            if last {
                break;
            }
        }

        if top_level {
            self.advance_with_docstring()?;
        } else {
            self.advance()?;
        }

        Ok(stmts)
    }

    fn parse_statement(&mut self) -> ParserResult<(Statement, bool)> {
        let mut last_statement_in_block = false;
        let mut must_have_semicolon_at_the_end = true;

        let statement = match self.next.unwrap() {
            Return => {
                self.advance()?; // `return`

                let expr = self.parse_expression(Precedence::Lowest.to_i8().unwrap())?;

                Ok(Statement::Return(expr))
            }
            Defer => {
                self.advance()?; // `defer`

                let expr = self.parse_expression(Precedence::Lowest.to_i8().unwrap())?;

                Ok(Statement::Defer(expr))
            }
            Var => {
                self.advance()?; // `var`

                let mut r#mut = None;

                if self.next.unwrap().is(Mut) {
                    r#mut = Some(self.current.span());

                    self.advance()?; // `mut`
                }

                let name = consume_ident!(self, "variable name in var statement");

                let mut r#type = None;

                if !self.next.unwrap().is(Assign) {
                    r#type = Some(self.parse_type()?);
                }

                consume!(self, Assign, "var statement");

                let value = self.parse_expression(Precedence::Lowest.to_i8().unwrap())?;

                Ok(Statement::Var(r#mut, name, r#type, value))
            }
            _ => {
                let expression = self.parse_expression(Precedence::Lowest.to_i8().unwrap())?;

                must_have_semicolon_at_the_end =
                    expression.unwrap().deref().must_have_semicolon_at_the_end();

                if !self.next.unwrap().is(Semicolon) && must_have_semicolon_at_the_end {
                    last_statement_in_block = true;
                }

                if last_statement_in_block || !must_have_semicolon_at_the_end {
                    Ok(Statement::ExpressionWithoutSemicolon(expression))
                } else {
                    Ok(Statement::Expression(expression))
                }
            }
        }?;

        if !last_statement_in_block && must_have_semicolon_at_the_end {
            consume!(self, Semicolon, "end of the statement");
        }

        Ok((statement, last_statement_in_block))
    }
}

#[cfg(test)]
mod statement_tests {
    use crate::{macros::parser_test, Parser};
    use string_interner::StringInterner;

    parser_test!(imut_var, "fun test() { var a = 3; }");
    parser_test!(mut_var, "fun test() { var mut a = 3; }");
    parser_test!(
        defer,
        "fun test() { var f = open(\"test\"); defer f.close(); }"
    );
    parser_test!(r#return, "fun test() i32 { return 2; }");
}
