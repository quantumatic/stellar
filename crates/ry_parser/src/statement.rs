use crate::{error::ParseError, macros::*, ParseResult, Parser};
use ry_ast::{
    precedence::Precedence,
    span::WithSpan,
    statement::*,
    token::{Keyword::*, Punctuator::*, RawToken::*},
    Mutability,
};

impl Parser<'_> {
    pub(crate) fn parse_statements_block(
        &mut self,
        top_level: bool,
    ) -> ParseResult<StatementsBlock> {
        consume!(self, Punctuator(OpenBrace), "statements block"); // `{`

        let mut stmts = vec![];

        loop {
            if let Punctuator(CloseBrace) = self.next.unwrap() {
                break;
            }

            let (stmt, last) = self.parse_statement()?;

            stmts.push(stmt);

            if last {
                break;
            }
        }

        if top_level {
            consume!(with_docstring self, Punctuator(CloseBrace), "end of the statement block");
        } else {
            consume!(self, Punctuator(CloseBrace), "end of the statement block");
        }

        Ok(stmts)
    }

    fn parse_statement(&mut self) -> ParseResult<(Statement, bool)> {
        let mut last_statement_in_block = false;
        let mut must_have_semicolon_at_the_end = true;

        let statement = match self.next.unwrap() {
            Keyword(Return) => {
                self.advance()?; // `return`

                ReturnStatement {
                    return_value: self.parse_expression(Precedence::Lowest)?,
                }
                .into()
            }
            Keyword(Defer) => {
                self.advance()?; // `defer`

                DeferStatement {
                    call: self.parse_expression(Precedence::Lowest)?,
                }
                .into()
            }
            Keyword(Var) => {
                self.advance()?; // `var`

                let mut mutability = Mutability::immutable();

                if let Keyword(Mut) = self.next.unwrap() {
                    mutability = Mutability::mutable(self.current.span());
                    self.advance()?; // `mut`
                }

                let name = consume_ident!(self, "variable name in var statement");

                let mut r#type = None;

                if let Punctuator(Colon) = self.next.unwrap() {
                    self.advance()?;
                    r#type = Some(self.parse_type()?);
                }

                consume!(self, Punctuator(Assign), "var statement");

                let value = self.parse_expression(Precedence::Lowest)?;

                VarStatement {
                    mutability,
                    name,
                    r#type,
                    value,
                }
                .into()
            }
            _ => {
                let expression = self.parse_expression(Precedence::Lowest)?;

                must_have_semicolon_at_the_end = !(*expression.unwrap()).with_block();

                match self.next.unwrap() {
                    Punctuator(Semicolon) => {}
                    _ => {
                        if must_have_semicolon_at_the_end {
                            last_statement_in_block = true;
                        }
                    }
                }

                if last_statement_in_block || !must_have_semicolon_at_the_end {
                    ExpressionStatement {
                        has_semicolon: false,
                        expression,
                    }
                    .into()
                } else {
                    ExpressionStatement {
                        has_semicolon: true,
                        expression,
                    }
                    .into()
                }
            }
        };

        if !last_statement_in_block && must_have_semicolon_at_the_end {
            consume!(self, Punctuator(Semicolon), "end of the statement");
        }

        Ok((statement, last_statement_in_block))
    }
}

#[cfg(test)]
mod statement_tests {
    use crate::{macros::parser_test, Parser};
    use ry_interner::Interner;

    parser_test!(imut_var, "fun test() { var a = 3; }");
    parser_test!(mut_var, "fun test() { var mut a = 3; }");
    parser_test!(
        defer,
        "fun test() { var f = open(\"test\"); defer f.close(); }"
    );
    parser_test!(r#return, "fun test(): i32 { return 2; }");
}
