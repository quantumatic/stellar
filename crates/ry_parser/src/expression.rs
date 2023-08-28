use ry_ast::{
    precedence::Precedence,
    token::{Keyword, Punctuator, RawToken},
    BinaryOperator, Expression, IdentifierAST, LambdaFunctionParameter, MatchExpressionItem,
    PostfixOperator, PrefixOperator, RawBinaryOperator, RawPostfixOperator, RawPrefixOperator,
    StructFieldExpression,
};

use crate::{
    diagnostics::UnexpectedTokenDiagnostic,
    list::ListParser,
    literal::LiteralParser,
    pattern::PatternParser,
    r#type::{TypeArgumentsParser, TypeParser},
    statement::StatementsBlockParser,
    Parse, ParseState,
};

/// Parser for Ry expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ExpressionParser {
    precedence: Precedence,
    prohibit_struct_expressions: bool,
}

impl ExpressionParser {
    /// Creates a parser for expressions with lowest precedence.
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a parser for expressions with specified precedence.
    #[inline(always)]
    #[must_use]
    pub const fn with_precedence(mut self, precedence: Precedence) -> Self {
        self.precedence = precedence;
        self
    }

    /// Creates a parser for expressions, that disallows struct expressions.
    #[inline(always)]
    #[must_use]
    pub const fn prohibit_struct_expressions(mut self, prohibit_struct_expressions: bool) -> Self {
        self.prohibit_struct_expressions = prohibit_struct_expressions;
        self
    }
}

impl Parse for ExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let mut left = PrimaryExpressionParser {
            prohibit_struct_expressions: self.prohibit_struct_expressions,
        }
        .parse(state)?;

        while self.precedence < state.next_token.raw.into() {
            left = match state.next_token.raw {
                RawToken::Punctuator(Punctuator::OpenParent) => {
                    CallExpressionParser { left }.parse(state)?
                }
                RawToken::Punctuator(Punctuator::Dot) => {
                    FieldAccessExpressionParser { left }.parse(state)?
                }
                RawToken::Punctuator(Punctuator::OpenBracket) => {
                    GenericArgumentsExpressionParser { left }.parse(state)?
                }
                RawToken::Keyword(Keyword::As) => CastExpressionParser { left }.parse(state)?,
                RawToken::Punctuator(Punctuator::OpenBrace) => {
                    if self.prohibit_struct_expressions {
                        return Some(left);
                    }

                    StructExpressionParser { left }.parse(state)?
                }
                _ => {
                    if state.next_token.raw.is_binary_operator() {
                        BinaryExpressionParser {
                            left,
                            prohibit_struct_expressions: self.prohibit_struct_expressions,
                        }
                        .parse(state)?
                    } else if state.next_token.raw.is_postfix_operator() {
                        PostfixExpressionParser { left }.parse(state)?
                    } else {
                        break;
                    }
                }
            };
        }

        Some(left)
    }
}

struct WhileExpressionParser;

impl Parse for WhileExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.location.start;
        state.advance(); // `while`

        let condition = ExpressionParser {
            precedence: Precedence::Lowest,
            prohibit_struct_expressions: true,
        }
        .parse(state)?;

        let body = StatementsBlockParser.parse(state)?;

        Some(Expression::While {
            location: state.location_from(start),
            condition: Box::new(condition),
            statements_block: body,
        })
    }
}

struct MatchExpressionParser;

impl Parse for MatchExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.location.start;
        state.advance(); // `match`

        let expression = ExpressionParser::new()
            .prohibit_struct_expressions(true)
            .parse(state)?;

        let block = MatchExpressionBlockParser.parse(state)?;

        Some(Expression::Match {
            location: state.location_from(start),
            expression: Box::new(expression),
            block,
        })
    }
}

struct MatchExpressionBlockParser;

impl Parse for MatchExpressionBlockParser {
    type Output = Option<Vec<MatchExpressionItem>>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.consume(Punctuator::OpenBrace)?;

        let units = ListParser::new(&[RawToken::from(Punctuator::CloseBrace)], |state| {
            MatchExpressionUnitParser.parse(state)
        })
        .parse(state)?;

        state.advance(); // `}`

        Some(units)
    }
}

struct MatchExpressionUnitParser;

impl Parse for MatchExpressionUnitParser {
    type Output = Option<MatchExpressionItem>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let left = PatternParser.parse(state)?;

        state.consume(Punctuator::Arrow)?;

        let right = ExpressionParser::new().parse(state)?;

        Some(MatchExpressionItem { left, right })
    }
}

struct PrimaryExpressionParser {
    prohibit_struct_expressions: bool,
}

impl Parse for PrimaryExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
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
                ParenthesizedOrTupleExpressionParser.parse(state)
            }
            RawToken::Punctuator(Punctuator::OpenBracket) => ListExpressionParser.parse(state),
            RawToken::Punctuator(Punctuator::OpenBrace) => {
                StatementsBlockExpressionParser.parse(state)
            }
            RawToken::Punctuator(Punctuator::Or) | RawToken::Punctuator(Punctuator::DoubleOr) => {
                LambdaExpressionParser.parse(state)
            }
            RawToken::Keyword(Keyword::If) => IfExpressionParser.parse(state),
            RawToken::Keyword(Keyword::Match) => MatchExpressionParser.parse(state),
            RawToken::Keyword(Keyword::While) => WhileExpressionParser.parse(state),
            RawToken::Keyword(Keyword::Loop) => LoopExpressionParser.parse(state),
            RawToken::Punctuator(Punctuator::Underscore) => {
                state.advance();

                Some(Expression::Underscore {
                    location: state.current_token.location,
                })
            }
            _ => {
                if state.next_token.raw.is_prefix_operator() {
                    return PrefixExpressionParser {
                        prohibit_struct_expressions: self.prohibit_struct_expressions,
                    }
                    .parse(state);
                }

                state.add_diagnostic(UnexpectedTokenDiagnostic::new(
                    state.current_token.location.end,
                    state.next_token,
                    "expression",
                ));
                None
            }
        }
    }
}

struct GenericArgumentsExpressionParser {
    left: Expression,
}

impl Parse for GenericArgumentsExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let arguments = TypeArgumentsParser.parse(state)?;

        Some(Expression::TypeArguments {
            location: state.location_from(self.left.location().start),
            left: Box::new(self.left),
            arguments,
        })
    }
}

struct FieldAccessExpressionParser {
    left: Expression,
}

impl Parse for FieldAccessExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance(); // `.`

        let right = state.consume_identifier()?;

        Some(Expression::FieldAccess {
            location: state.location_from(self.left.location().start),
            left: Box::new(self.left),
            right,
        })
    }
}

struct PrefixExpressionParser {
    prohibit_struct_expressions: bool,
}
impl Parse for PrefixExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let operator_token = state.next_token;
        let operator: PrefixOperator = PrefixOperator {
            location: operator_token.location,
            raw: RawPrefixOperator::from(operator_token.raw),
        };
        state.advance();

        let inner = ExpressionParser {
            precedence: Precedence::Unary,
            prohibit_struct_expressions: self.prohibit_struct_expressions,
        }
        .parse(state)?;

        Some(Expression::Prefix {
            location: state.make_location(operator_token.location.start, inner.location().end),
            inner: Box::new(inner),
            operator,
        })
    }
}

struct PostfixExpressionParser {
    left: Expression,
}

impl Parse for PostfixExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let operator: PostfixOperator = PostfixOperator {
            location: state.current_token.location,
            raw: RawPostfixOperator::from(state.current_token.raw),
        };

        Some(Expression::Postfix {
            location: state.location_from(self.left.location().start),
            inner: Box::new(self.left),
            operator,
        })
    }
}

struct ParenthesizedOrTupleExpressionParser;

impl Parse for ParenthesizedOrTupleExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
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
}

struct IfExpressionParser;

impl Parse for IfExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.location.start;
        state.advance(); // `if`

        let condition = ExpressionParser {
            precedence: Precedence::Lowest,
            prohibit_struct_expressions: true,
        }
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

            let condition = ExpressionParser {
                precedence: Precedence::Lowest,
                prohibit_struct_expressions: true,
            }
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
}

struct CastExpressionParser {
    left: Expression,
}

impl Parse for CastExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance();

        let right = TypeParser.parse(state)?;

        Some(Expression::As {
            location: state.location_from(self.left.location().start),
            left: Box::new(self.left),
            right,
        })
    }
}

struct CallExpressionParser {
    left: Expression,
}

impl Parse for CallExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance(); // `(`

        let arguments = ListParser::new(&[RawToken::from(Punctuator::CloseParent)], |state| {
            ExpressionParser::default().parse(state)
        })
        .parse(state)?;

        state.advance();

        Some(Expression::Call {
            location: state.location_from(self.left.location().start),
            callee: Box::new(self.left),
            arguments,
        })
    }
}

struct BinaryExpressionParser {
    left: Expression,
    prohibit_struct_expressions: bool,
}

impl Parse for BinaryExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let operator_token = state.next_token;
        let operator: BinaryOperator = BinaryOperator {
            location: operator_token.location,
            raw: RawBinaryOperator::from(operator_token.raw),
        };
        let precedence = state.next_token.raw.into();

        state.advance();

        let right = ExpressionParser {
            precedence,
            prohibit_struct_expressions: self.prohibit_struct_expressions,
        }
        .parse(state)?;

        Some(Expression::Binary {
            location: state.location_from(self.left.location().start),
            left: Box::new(self.left),
            right: Box::new(right),
            operator,
        })
    }
}

struct ListExpressionParser;

impl Parse for ListExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
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
}

struct StructExpressionParser {
    left: Expression,
}

impl Parse for StructExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance(); // `{`

        let fields = ListParser::new(&[RawToken::from(Punctuator::CloseBrace)], |state| {
            StructFieldExpressionParser.parse(state)
        })
        .parse(state)?;

        state.advance(); // `}`

        Some(Expression::Struct {
            location: state.location_from(self.left.location().start),
            left: Box::new(self.left),
            fields,
        })
    }
}

struct StructFieldExpressionParser;

impl Parse for StructFieldExpressionParser {
    type Output = Option<StructFieldExpression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let name = state.consume_identifier()?;

        let value = if state.next_token.raw == Punctuator::Colon {
            state.advance();
            Some(ExpressionParser::default().parse(state)?)
        } else {
            None
        };

        Some(StructFieldExpression { name, value })
    }
}

struct StatementsBlockExpressionParser;

impl Parse for StatementsBlockExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.location.start;
        let block = StatementsBlockParser.parse(state)?;

        Some(Expression::StatementsBlock {
            location: state.location_from(start),
            block,
        })
    }
}

struct LambdaExpressionParser;

impl Parse for LambdaExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        let start = state.next_token.location.start;

        state.advance(); // `|` or `||`

        let parameters = if state.current_token.raw == Punctuator::Or {
            ListParser::new(&[RawToken::from(Punctuator::Or)], |state| {
                let name = state.consume_identifier()?;

                let ty = if state.next_token.raw == Punctuator::Colon {
                    state.advance();

                    Some(TypeParser.parse(state)?)
                } else {
                    None
                };

                Some(LambdaFunctionParameter { name, ty })
            })
            .parse(state)?
        } else {
            vec![]
        };

        state.advance();

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
}

struct LoopExpressionParser;

impl Parse for LoopExpressionParser {
    type Output = Option<Expression>;

    fn parse(self, state: &mut ParseState<'_, '_, '_>) -> Self::Output {
        state.advance(); // `loop`

        let location = state.current_token.location;
        let statements_block = StatementsBlockParser.parse(state)?;

        Some(Expression::Loop {
            location,
            statements_block,
        })
    }
}
