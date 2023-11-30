use stellar_ast::{
    precedence::Precedence,
    token::{Keyword, Punctuator, RawToken},
    BinaryOperator, Expression, IdentifierAST, LambdaFunctionParameter, MatchExpressionItem,
    PostfixOperator, PrefixOperator, RawBinaryOperator, RawPostfixOperator, RawPrefixOperator,
    StructFieldExpression,
};
use stellar_english_commons::enumeration::one_of;

use crate::{
    list::ListParser,
    literal::LiteralParser,
    pattern::PatternParser,
    r#type::{TypeArgumentsParser, TypeParser},
    statement::StatementsBlockParser,
    Parse, ParseState,
};

/// Parser for Stellar expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ExpressionParser {
    in_statements_block: bool,
    precedence: Precedence,
    prohibit_struct_expressions: bool,
}

impl ExpressionParser {
    /// Creates a parser for expressions with lowest precedence.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a parser for expressions with specified precedence.
    #[inline]
    #[must_use]
    pub const fn with_precedence(mut self, precedence: Precedence) -> Self {
        self.precedence = precedence;
        self
    }

    /// Creates a parser for expressions, that disallows struct expressions.
    #[inline]
    #[must_use]
    pub const fn prohibit_struct_expressions(mut self) -> Self {
        self.prohibit_struct_expressions = true;
        self
    }

    /// Creates a parser for expressions, that disallows struct expressions if
    /// condition is satisfied.
    #[inline]
    #[must_use]
    pub const fn prohibit_struct_expressions_if(mut self, condition: bool) -> Self {
        self.prohibit_struct_expressions = condition;
        self
    }

    /// Creates a parser for expressions, that prints diagnostics
    /// specific to statements blocks.
    #[inline]
    #[must_use]
    pub const fn in_statements_block(mut self) -> Self {
        self.in_statements_block = true;
        self
    }

    fn parse_call_expression(
        self,
        state: &mut ParseState<'_, '_>,
        left: Expression,
    ) -> Option<Expression> {
        state.advance(); // `(`

        let arguments = ListParser::new(&[RawToken::from(Punctuator::CloseParent)], |state| {
            ExpressionParser::default().parse(state)
        })
        .parse(state)?;

        state.advance();

        Some(Expression::Call {
            location: state.location_from(left.location().start),
            callee: Box::new(left),
            arguments,
        })
    }

    fn parse_field_access_expression(
        self,
        state: &mut ParseState<'_, '_>,
        left: Expression,
    ) -> Option<Expression> {
        state.advance(); // `.`

        let right = state.consume_identifier()?;

        Some(Expression::FieldAccess {
            location: state.location_from(left.location().start),
            left: Box::new(left),
            right,
        })
    }

    fn parse_type_arguments_expression(
        self,
        state: &mut ParseState<'_, '_>,
        left: Expression,
    ) -> Option<Expression> {
        let arguments = TypeArgumentsParser.parse(state)?;

        Some(Expression::TypeArguments {
            location: state.location_from(left.location().start),
            left: Box::new(left),
            arguments,
        })
    }

    fn parse_cast_expression(
        self,
        state: &mut ParseState<'_, '_>,
        left: Expression,
    ) -> Option<Expression> {
        state.advance();

        let right = TypeParser.parse(state)?;

        Some(Expression::As {
            location: state.location_from(left.location().start),
            left: Box::new(left),
            right,
        })
    }

    fn parse_struct_field_expression(
        self,
        state: &mut ParseState<'_, '_>,
    ) -> Option<StructFieldExpression> {
        let name = state.consume_identifier()?;

        let value = if state.next_token.raw == Punctuator::Colon {
            state.advance();
            Some(ExpressionParser::default().parse(state)?)
        } else {
            None
        };

        Some(StructFieldExpression { name, value })
    }

    fn parse_struct_expression(
        self,
        state: &mut ParseState<'_, '_>,
        left: Expression,
    ) -> Option<Expression> {
        state.advance(); // `{`

        let fields = ListParser::new(&[RawToken::from(Punctuator::CloseBrace)], |state| {
            self.parse_struct_field_expression(state)
        })
        .parse(state)?;

        state.advance(); // `}`

        Some(Expression::Struct {
            location: state.location_from(left.location().start),
            left: Box::new(left),
            fields,
        })
    }

    fn parse_binary_expression(
        self,
        state: &mut ParseState<'_, '_>,
        left: Expression,
    ) -> Option<Expression> {
        let operator_token = state.next_token;
        let operator: BinaryOperator = BinaryOperator {
            location: operator_token.location,
            raw: RawBinaryOperator::from(operator_token.raw),
        };
        let precedence = state.next_token.raw.into();

        state.advance();

        let right = ExpressionParser::new()
            .with_precedence(precedence)
            .prohibit_struct_expressions_if(self.prohibit_struct_expressions)
            .parse(state)?;

        Some(Expression::Binary {
            location: state.location_from(left.location().start),
            left: Box::new(left),
            right: Box::new(right),
            operator,
        })
    }

    fn parse_postfix_expression(
        self,
        state: &mut ParseState<'_, '_>,
        left: Expression,
    ) -> Option<Expression> {
        state.advance();

        let operator: PostfixOperator = PostfixOperator {
            location: state.current_token.location,
            raw: RawPostfixOperator::from(state.current_token.raw),
        };

        Some(Expression::Postfix {
            location: state.location_from(left.location().start),
            inner: Box::new(left),
            operator,
        })
    }
}

impl Parse for ExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        let mut left = PrimaryExpressionParser {
            in_statements_block: self.in_statements_block,
            prohibit_struct_expressions: self.prohibit_struct_expressions,
        }
        .parse(state)?;

        while self.precedence < state.next_token.raw.into() && !left.with_block() {
            left = match state.next_token.raw {
                RawToken::Punctuator(Punctuator::OpenParent) => {
                    self.parse_call_expression(state, left)
                }
                RawToken::Punctuator(Punctuator::Dot) => {
                    self.parse_field_access_expression(state, left)
                }
                RawToken::Punctuator(Punctuator::OpenBracket) => {
                    self.parse_type_arguments_expression(state, left)
                }
                RawToken::Keyword(Keyword::As) => self.parse_cast_expression(state, left),
                RawToken::Punctuator(Punctuator::OpenBrace) => {
                    if self.prohibit_struct_expressions {
                        return Some(left);
                    }

                    self.parse_struct_expression(state, left)
                }
                _ => {
                    if state.next_token.raw.is_binary_operator() {
                        self.parse_binary_expression(state, left)
                    } else if state.next_token.raw.is_postfix_operator() {
                        self.parse_postfix_expression(state, left)
                    } else {
                        break;
                    }
                }
            }?;
        }

        Some(left)
    }
}

struct PrimaryExpressionParser {
    in_statements_block: bool,
    prohibit_struct_expressions: bool,
}

impl PrimaryExpressionParser {
    fn parse_parenthesized_or_tuple_expression(
        self,
        state: &mut ParseState<'_, '_>,
    ) -> Option<Expression> {
        let start = state.next_token.location.start;
        state.advance();

        let elements = ListParser::new(&[RawToken::from(Punctuator::CloseParent)], |state| {
            ExpressionParser::default().parse(state)
        })
        .parse(state)?;

        state.advance(); // `)`

        let location = state.location_from(start);

        let mut elements = elements.into_iter();

        match (elements.next(), elements.next()) {
            (Some(element), None) => {
                if state
                    .resolve_location(state.location_from(element.location().end))
                    .contains(',')
                {
                    Some(Expression::Tuple {
                        location,
                        elements: vec![element],
                    })
                } else {
                    Some(Expression::Parenthesized {
                        location,
                        inner: Box::from(element),
                    })
                }
            }
            (None, None) => Some(Expression::Tuple {
                location,
                elements: vec![],
            }),
            (Some(previous), Some(next)) => {
                let mut new_elements = vec![];
                new_elements.push(previous);
                new_elements.push(next);

                new_elements.append(&mut elements.collect::<Vec<_>>());

                Some(Expression::Tuple {
                    location,
                    elements: new_elements,
                })
            }
            _ => unreachable!(),
        }
    }

    fn parse_list_expression(&self, state: &mut ParseState<'_, '_>) -> Option<Expression> {
        let start = state.next_token.location.start;

        state.advance();

        let elements = ListParser::new(&[RawToken::from(Punctuator::CloseBracket)], |state| {
            ExpressionParser::default().parse(state)
        })
        .parse(state)?;

        state.advance();

        Some(Expression::List {
            location: state.location_from(start),
            elements,
        })
    }

    fn parse_block_expression(&self, state: &mut ParseState<'_, '_>) -> Option<Expression> {
        let start = state.next_token.location.start;
        let block = StatementsBlockParser.parse(state)?;

        Some(Expression::StatementsBlock {
            location: state.location_from(start),
            block,
        })
    }

    fn parse_lambda_expression(&self, state: &mut ParseState<'_, '_>) -> Option<Expression> {
        let start = state.next_token.location.start;

        state.advance(); // `|` or `||`

        let parameters = if state.current_token.raw == Punctuator::Or {
            let parameters = ListParser::new(&[RawToken::from(Punctuator::Or)], |state| {
                let name = state.consume_identifier()?;

                let ty = if state.next_token.raw == Punctuator::Colon {
                    state.advance();

                    Some(TypeParser.parse(state)?)
                } else {
                    None
                };

                Some(LambdaFunctionParameter { name, ty })
            })
            .parse(state)?;

            state.advance();

            parameters
        } else {
            vec![]
        };

        let return_type = if state.next_token.raw == Punctuator::Colon {
            state.advance();

            Some(TypeParser.parse(state)?)
        } else {
            None
        };

        let value = ExpressionParser::default().parse(state)?;

        Some(Expression::Lambda {
            location: state.location_from(start),
            parameters,
            return_type,
            value: Box::new(value),
        })
    }

    fn parse_if_expression(&self, state: &mut ParseState<'_, '_>) -> Option<Expression> {
        let start = state.next_token.location.start;
        state.advance(); // `if`

        let condition = ExpressionParser::new()
            .prohibit_struct_expressions()
            .parse(state)?;

        let block = StatementsBlockParser.parse(state)?;

        let mut if_blocks = vec![(condition, block)];

        let mut r#else = None;

        while state.next_token.raw == Keyword::Else {
            state.advance();

            if state.next_token.raw != Keyword::If {
                r#else = Some(StatementsBlockParser.parse(state)?);
                break;
            }

            state.advance();

            let condition = ExpressionParser::new()
                .prohibit_struct_expressions()
                .parse(state)?;
            let block = StatementsBlockParser.parse(state)?;

            if_blocks.push((condition, block));
        }

        Some(Expression::If {
            location: state.location_from(start),
            if_blocks,
            r#else,
        })
    }

    fn parse_match_expression_item(
        &self,
        state: &mut ParseState<'_, '_>,
    ) -> Option<MatchExpressionItem> {
        let left = PatternParser.parse(state)?;

        state.consume(Punctuator::Arrow)?;

        let right = ExpressionParser::new().parse(state)?;

        Some(MatchExpressionItem { left, right })
    }

    fn parse_match_expression_block(
        &self,
        state: &mut ParseState<'_, '_>,
    ) -> Option<Vec<MatchExpressionItem>> {
        state.consume(Punctuator::OpenBrace)?;

        let items = ListParser::new(&[RawToken::from(Punctuator::CloseBrace)], |state| {
            self.parse_match_expression_item(state)
        })
        .parse(state)?;

        state.advance(); // `}`

        Some(items)
    }

    fn parse_match_expression(&self, state: &mut ParseState<'_, '_>) -> Option<Expression> {
        let start = state.next_token.location.start;
        state.advance(); // `match`

        let expression = ExpressionParser::new()
            .prohibit_struct_expressions()
            .parse(state)?;

        let block = self.parse_match_expression_block(state)?;

        Some(Expression::Match {
            location: state.location_from(start),
            expression: Box::new(expression),
            block,
        })
    }

    fn parse_while_expression(&self, state: &mut ParseState<'_, '_>) -> Option<Expression> {
        let start = state.next_token.location.start;
        state.advance(); // `while`

        let condition = ExpressionParser::new()
            .prohibit_struct_expressions()
            .parse(state)?;

        let body = StatementsBlockParser.parse(state)?;

        Some(Expression::While {
            location: state.location_from(start),
            condition: Box::new(condition),
            statements_block: body,
        })
    }

    fn parse_loop_expression(&self, state: &mut ParseState<'_, '_>) -> Option<Expression> {
        state.advance(); // `loop`

        let location = state.current_token.location;
        let statements_block = StatementsBlockParser.parse(state)?;

        Some(Expression::Loop {
            location,
            statements_block,
        })
    }

    fn parse_prefix_expression(&self, state: &mut ParseState<'_, '_>) -> Option<Expression> {
        let operator_token = state.next_token;
        let operator: PrefixOperator = PrefixOperator {
            location: operator_token.location,
            raw: RawPrefixOperator::from(operator_token.raw),
        };
        state.advance();

        let inner = ExpressionParser::new()
            .with_precedence(Precedence::Unastellar)
            .prohibit_struct_expressions_if(self.prohibit_struct_expressions)
            .parse(state)?;

        Some(Expression::Prefix {
            location: state.make_location(operator_token.location.start, inner.location().end),
            inner: Box::new(inner),
            operator,
        })
    }
}

impl Parse for PrimaryExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_>) -> Self::Output {
        match state.next_token.raw {
            RawToken::IntegerLiteral
            | RawToken::FloatLiteral
            | RawToken::StringLiteral
            | RawToken::CharLiteral
            | RawToken::TrueBoolLiteral
            | RawToken::FalseBoolLiteral => Some(Expression::Literal(LiteralParser.parse(state)?)),
            RawToken::Identifier => {
                let symbol = state.lexer.scanned_identifier;
                state.advance();

                Some(Expression::Identifier(IdentifierAST {
                    location: state.current_token.location,
                    id: symbol,
                }))
            }
            RawToken::Punctuator(Punctuator::OpenParent) => {
                self.parse_parenthesized_or_tuple_expression(state)
            }
            RawToken::Punctuator(Punctuator::OpenBracket) => self.parse_list_expression(state),
            RawToken::Punctuator(Punctuator::OpenBrace) => self.parse_block_expression(state),
            RawToken::Punctuator(Punctuator::Or) | RawToken::Punctuator(Punctuator::DoubleOr) => {
                self.parse_lambda_expression(state)
            }
            RawToken::Keyword(Keyword::If) => self.parse_if_expression(state),
            RawToken::Keyword(Keyword::Match) => self.parse_match_expression(state),
            RawToken::Keyword(Keyword::While) => self.parse_while_expression(state),
            RawToken::Keyword(Keyword::Loop) => self.parse_loop_expression(state),
            RawToken::Punctuator(Punctuator::Underscore) => {
                state.advance();

                Some(Expression::Underscore {
                    location: state.current_token.location,
                })
            }
            _ => {
                if state.next_token.raw.is_prefix_operator() {
                    return self.parse_prefix_expression(state);
                }

                if self.in_statements_block {
                    state.add_unexpected_token_diagnostic(one_of([
                        "statement".to_owned(),
                        Punctuator::Semicolon.to_string(),
                        Punctuator::CloseBrace.to_string(),
                    ]));
                } else {
                    state.add_unexpected_token_diagnostic("expression");
                }

                None
            }
        }
    }
}
