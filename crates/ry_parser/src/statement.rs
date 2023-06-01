use crate::{expression::ExpressionParser, Cursor, Parse};
use ry_ast::{span::Span, token::RawToken, Statement, StatementsBlock, Token};
use ry_diagnostics::{parser::ParseDiagnostic, Report};

struct StatementParser;

pub(crate) struct StatementsBlockParser;

struct DeferStatementParser;

struct ReturnStatementParser;

impl Parse for StatementParser {
    type Output = Option<(Statement, bool)>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let mut last_statement_in_block = false;
        let mut must_have_semicolon_at_the_end = true;

        let mut no_semicolon_after_expression_error_emitted = false;

        let start = cursor.next.span().start();

        let statement = match cursor.next.unwrap() {
            Token![return] => ReturnStatementParser.parse_with(cursor)?,
            Token![defer] => DeferStatementParser.parse_with(cursor)?,
            _ => {
                let expression = ExpressionParser::default().parse_with(cursor)?;

                must_have_semicolon_at_the_end = !expression.unwrap().with_block();

                match cursor.next.unwrap() {
                    Token![;] => {}
                    Token!['}'] => {
                        if must_have_semicolon_at_the_end {
                            last_statement_in_block = true;
                        }
                    }
                    _ => {
                        no_semicolon_after_expression_error_emitted = true;

                        cursor.diagnostics.push(
                            ParseDiagnostic::NoSemicolonAfterExpressionError {
                                expression_location: expression.span(),
                                at: Span::new(
                                    expression.span().end() - 1,
                                    expression.span().end(),
                                    cursor.file_id,
                                ),
                            }
                            .build(),
                        );
                    }
                }

                if last_statement_in_block || !must_have_semicolon_at_the_end {
                    Statement::Expression {
                        has_semicolon: false,
                        expression,
                    }
                } else {
                    Statement::Expression {
                        has_semicolon: true,
                        expression,
                    }
                }
            }
        };

        let end = cursor.current.span().end();

        if !last_statement_in_block && must_have_semicolon_at_the_end {
            if cursor.next.unwrap() != &Token![;] {
                if !no_semicolon_after_expression_error_emitted {
                    cursor.diagnostics.push(
                        ParseDiagnostic::NoSemicolonAfterStatementError {
                            statement_location: Span::new(start, end - 1, cursor.file_id),
                            at: Span::new(end, end, cursor.file_id),
                        }
                        .build(),
                    );
                }
            } else {
                cursor.next_token();
            }
        }

        Some((statement, last_statement_in_block))
    }
}

impl Parse for StatementsBlockParser {
    type Output = Option<StatementsBlock>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.consume(Token!['{'], "statements block")?;
        let start = cursor.current.span().start();

        let mut block = vec![];

        loop {
            match cursor.next.unwrap() {
                Token!['}'] => break,
                RawToken::EndOfFile => {
                    cursor.diagnostics.push(
                        ParseDiagnostic::EOFInsteadOfCloseBraceForStatementsBlockError {
                            statements_block_start_location: Span::new(
                                start,
                                start + 1,
                                cursor.file_id,
                            ),
                            at: cursor.current.span(),
                        }
                        .build(),
                    );

                    return None;
                }
                Token![;] => {
                    cursor.diagnostics.push(
                        ParseDiagnostic::EmptyStatementError {
                            at: cursor.next.span(),
                        }
                        .build(),
                    );

                    cursor.next_token();
                    continue;
                }
                _ => {}
            }

            let (statement, last) = StatementParser.parse_with(cursor)?;
            block.push(statement);

            if last {
                break;
            }
        }

        cursor.next_token();

        Some(block)
    }
}

impl Parse for DeferStatementParser {
    type Output = Option<Statement>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token();

        Some(Statement::Defer {
            call: ExpressionParser::default().parse_with(cursor)?,
        })
    }
}

impl Parse for ReturnStatementParser {
    type Output = Option<Statement>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token();

        Some(Statement::Return {
            expression: ExpressionParser::default().parse_with(cursor)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::StatementParser;
    use crate::macros::parse_test;

    parse_test!(StatementParser, r#return, "return a?.b.unwrap_or(0);");
    parse_test!(StatementParser, defer, "defer call();");
}