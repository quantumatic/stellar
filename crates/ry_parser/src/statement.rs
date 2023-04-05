use crate::{ParseResult, Parser};
use ry_ast::{
    precedence::Precedence,
    statement::*,
    token::{Keyword::*, Punctuator::*, RawToken::*},
    Mutability,
};

impl Parser<'_> {
    pub(crate) fn parse_statements_block(
        &mut self,
        top_level: bool,
    ) -> ParseResult<StatementsBlock> {
        self.consume(Punctuator(OpenBrace), "statements block")?;

        let mut block = vec![];

        while self.next.inner != Punctuator(CloseBrace) {
            let (statement, last) = self.parse_statement()?;
            block.push(statement);

            if last {
                break;
            }
        }

        if top_level {
            self.consume_with_docstring(Punctuator(CloseBrace), "end of the statements block")?;
        } else {
            self.consume(Punctuator(CloseBrace), "end of the statements block")?;
        }

        Ok(block)
    }

    fn parse_statement(&mut self) -> ParseResult<(Statement, bool)> {
        let mut last_statement_in_block = false;
        let mut must_have_semicolon_at_the_end = true;

        let statement = match self.next.inner {
            Keyword(Return) => {
                self.advance();

                ReturnStatement {
                    return_value: self.parse_expression(Precedence::Lowest)?,
                }
                .into()
            }
            Keyword(Defer) => {
                self.advance();

                DeferStatement {
                    call: self.parse_expression(Precedence::Lowest)?,
                }
                .into()
            }
            Keyword(Var) => {
                self.advance();

                let mut mutability = Mutability::immutable();

                if let Keyword(Mut) = self.next.inner {
                    mutability = Mutability::mutable(self.current.span);
                    self.advance();
                }

                let name = self.consume_identifier("variable name in var statement")?;

                let mut r#type = None;

                if self.next.inner == Punctuator(Colon) {
                    self.advance();
                    r#type = Some(self.parse_type()?);
                }

                self.consume(Punctuator(Assign), "var statement")?;

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

                must_have_semicolon_at_the_end = !expression.inner.with_block();

                match self.next.inner {
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
            self.consume(Punctuator(Semicolon), "end of the statement")?;
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
