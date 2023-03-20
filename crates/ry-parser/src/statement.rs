use crate::{error::ParserError, macros::*, Parser, ParserResult};
use num_traits::ToPrimitive;
use ry_ast::{precedence::Precedence, token::RawToken::*, *};
use std::ops::Deref;

impl<'c> Parser<'c> {
    pub(crate) fn parse_statements_block(
        &mut self,
        top_level: bool,
    ) -> ParserResult<StatementsBlock> {
        check_token!(self, OpenBrace, "statements block")?;

        self.advance(false)?; // `{`

        let mut stmts = vec![];

        while !self.current.value.is(CloseBrace) {
            let (stmt, last) = self.parse_statement()?;

            stmts.push(stmt);

            if last {
                break;
            }
        }

        check_token!(self, CloseBrace, "statements block")?;

        if top_level {
            self.advance(true)?;
        } else {
            self.advance(false)?;
        }

        Ok(stmts)
    }

    fn parse_statement(&mut self) -> ParserResult<(Statement, bool)> {
        let mut last_statement_in_block = false;
        let mut must_have_semicolon_at_the_end = true;

        let statement = match self.current.value {
            Return => {
                self.advance(false)?; // `return`

                let expr = self.parse_expression(Precedence::Lowest.to_i8().unwrap())?;

                Ok(Statement::Return(expr))
            }
            Defer => {
                self.advance(false)?; // `defer`

                let expr = self.parse_expression(Precedence::Lowest.to_i8().unwrap())?;

                Ok(Statement::Defer(expr))
            }
            Var => {
                self.advance(false)?; // `var`

                let mut r#mut = None;

                if self.current.value.is(Mut) {
                    r#mut = Some(self.current.span);

                    self.advance(false)?; // `mut`
                }

                check_token0!(self, "identifier", Identifier(_), "var statement")?;

                let name = self.current_ident_with_span();

                self.advance(false)?; // id

                let mut r#type = None;

                if !self.current.value.is(Assign) {
                    r#type = Some(self.parse_type()?);
                }

                check_token!(self, Assign, "var statement")?;

                self.advance(false)?; // `=`

                let value = self.parse_expression(Precedence::Lowest.to_i8().unwrap())?;

                Ok(Statement::Var(r#mut, name, r#type, value))
            }
            _ => {
                let expression = self.parse_expression(Precedence::Lowest.to_i8().unwrap())?;

                must_have_semicolon_at_the_end =
                    expression.value.deref().must_have_semicolon_at_the_end();

                if !self.current.value.is(Semicolon) && must_have_semicolon_at_the_end {
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
            check_token!(self, Semicolon, "end of the statement")?;
            self.advance(false)?; // `;`
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
