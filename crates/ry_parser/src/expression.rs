use crate::{
    items::FunctionParameterParser,
    literal::LiteralParser,
    macros::parse_list,
    pattern::PatternParser,
    r#type::{GenericArgumentsParser, TypeParser},
    statement::StatementsBlockParser,
    Cursor, Parse,
};
use ry_ast::{
    precedence::Precedence, token::RawToken, BinaryOperator, Expression, MatchExpressionUnit,
    PostfixOperator, PrefixOperator, StructExpressionUnit, Token,
};
use ry_diagnostics::{expected, parser::ParseDiagnostic, Report};
use ry_span::{At, Span, Spanned};

#[derive(Default)]
pub(crate) struct ExpressionParser {
    pub(crate) precedence: Precedence,
    pub(crate) ignore_struct: bool,
}

struct WhileExpressionParser;

struct MatchExpressionParser;

struct MatchExpressionBlockParser;

struct MatchExpressionUnitParser;

struct PrimaryExpressionParser {
    pub(crate) ignore_struct: bool,
}

struct GenericArgumentsExpressionParser {
    pub(crate) left: Spanned<Expression>,
}

struct PropertyAccessExpressionParser {
    pub(crate) left: Spanned<Expression>,
}

struct PrefixExpressionParser {
    pub(crate) ignore_struct: bool,
}

struct PostfixExpressionParser {
    pub(crate) left: Spanned<Expression>,
}

struct CastExpressionParser {
    pub(crate) left: Spanned<Expression>,
}

struct IfExpressionParser;

struct ParenthesizedExpressionParser;

struct CallExpressionParser {
    pub(crate) left: Spanned<Expression>,
}

struct BinaryExpressionParser {
    pub(crate) left: Spanned<Expression>,
    pub(crate) ignore_struct: bool,
}

struct ArrayLiteralExpressionParser;

struct TupleExpressionParser;

struct StructExpressionParser {
    pub(crate) left: Spanned<Expression>,
}

struct StructExpressionUnitParser;

struct FunctionExpressionParser;

struct StatementsBlockExpressionParser;

impl ExpressionParser {
    pub fn ignore_struct(&mut self) {
        self.ignore_struct = true;
    }
}

impl Parse for ExpressionParser {
    type Output = Option<Spanned<Expression>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let mut left = PrimaryExpressionParser {
            ignore_struct: self.ignore_struct
        }.parse_with(cursor)?;

        while self.precedence < cursor.next.unwrap().to_precedence() {
            left = match cursor.next.unwrap() {
                Token!['('] => CallExpressionParser { left }.parse_with(cursor)?,
                Token![.] => PropertyAccessExpressionParser { left }.parse_with(cursor)?,
                Token!['['] => GenericArgumentsExpressionParser { left }.parse_with(cursor)?,
                Token![as] => CastExpressionParser { left }.parse_with(cursor)?,
                Token!['{'] => {
                    if self.ignore_struct {
                        return Some(left);
                    } else {
                        StructExpressionParser { left }.parse_with(cursor)?
                    }
                },
                _ => {
                    if cursor.next.unwrap().binary_operator() {
                        BinaryExpressionParser { left, ignore_struct: self.ignore_struct }.parse_with(cursor)?
                    } else if cursor.next.unwrap().postfix_operator() {
                        PostfixExpressionParser { left }.parse_with(cursor)?
                    } else {
                        break;
                    }
                }
            };
        }

        Some(left)
    }
}

impl Parse for WhileExpressionParser {
    type Output = Option<Spanned<Expression>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token(); // `while`
        let start = cursor.current.span().start();

        let mut condition_parser = ExpressionParser::default();
        condition_parser.ignore_struct();
        let condition = condition_parser.parse_with(cursor)?;

        let body = StatementsBlockParser.parse_with(cursor)?;

        Some(
            Expression::While {
                condition: Box::new(condition),
                body,
            }
            .at(Span::new(
                start,
                cursor.current.span().end(),
                cursor.file_id,
            )),
        )
    }
}

impl Parse for MatchExpressionParser {
    type Output = Option<Spanned<Expression>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token(); // `match`

        let mut expression_parser = ExpressionParser::default();
        expression_parser.ignore_struct();
        let expression = expression_parser.parse_with(cursor)?;

        let block = MatchExpressionBlockParser.parse_with(cursor)?;

        let span = Span::new(
            expression.span().start(),
            cursor.current.span().end(),
            cursor.file_id,
        );
        Some(
            Expression::Match {
                expression: Box::new(expression),
                block,
            }
            .at(span),
        )
    }
}

impl Parse for MatchExpressionBlockParser {
    type Output = Option<Vec<MatchExpressionUnit>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.consume(Token!['{'], "match expression block")?;

        let units = parse_list!(cursor, "match expression block", Token!['}'], || {
            MatchExpressionUnitParser.parse_with(cursor)
        });

        cursor.next_token(); // `}`

        Some(units)
    }
}

impl Parse for MatchExpressionUnitParser {
    type Output = Option<MatchExpressionUnit>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let left = PatternParser.parse_with(cursor)?;
        cursor.consume(Token![=>], "match expression unit")?;

        let right = ExpressionParser::default().parse_with(cursor)?;

        Some(MatchExpressionUnit { left, right })
    }
}

impl Parse for PrimaryExpressionParser {
    type Output = Option<Spanned<Expression>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        match *cursor.next.unwrap() {
            RawToken::IntegerLiteral
            | RawToken::FloatLiteral
            | RawToken::StringLiteral
            | RawToken::CharLiteral
            | Token![true]
            | Token![false] => {
                let literal = LiteralParser.parse_with(cursor)?;
                let span = literal.span();

                Some(Expression::Literal(literal.take()).at(span))
            }
            RawToken::Identifier => {
                let symbol = cursor.lexer.identifier();
                cursor.next_token();
                Some(Expression::Identifier(symbol).at(cursor.current.span()))
            }
            Token!['('] => ParenthesizedExpressionParser.parse_with(cursor),
            Token!['['] => ArrayLiteralExpressionParser.parse_with(cursor),
            Token!['{'] => StatementsBlockExpressionParser.parse_with(cursor),
            Token![|] => FunctionExpressionParser.parse_with(cursor),
            Token![#] => TupleExpressionParser.parse_with(cursor),
            Token![if] => IfExpressionParser.parse_with(cursor),
            Token![match] => MatchExpressionParser.parse_with(cursor),
            Token![while] => WhileExpressionParser.parse_with(cursor),
            _ => {
                if cursor.next.unwrap().prefix_operator() {
                    return PrefixExpressionParser { ignore_struct: self.ignore_struct }.parse_with(cursor);
                }
                cursor.diagnostics.push(
                    ParseDiagnostic::UnexpectedTokenError {
                        got: cursor.next.clone(),
                        expected: expected!(
                            "integer literal",
                            "float literal",
                            "string literal",
                            "char literal",
                            "boolean literal",
                            Token![#],
                            Token![|],
                            Token!['('],
                            Token!['{'],
                            Token!['['],
                            "identifier",
                            Token![if],
                            Token![while],
                            Token![match]
                        ),
                        node: "expression".to_owned(),
                    }
                    .build(),
                );
                None
            }
        }
    }
}

impl Parse for GenericArgumentsExpressionParser {
    type Output = Option<Spanned<Expression>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let arguments = GenericArgumentsParser.parse_with(cursor)?;

        let span = Span::new(
            self.left.span().start(),
            cursor.current.span().end(),
            cursor.file_id,
        );

        Some(
            Expression::GenericArguments {
                left: Box::new(self.left),
                arguments,
            }
            .at(span),
        )
    }
}

impl Parse for PropertyAccessExpressionParser {
    type Output = Option<Spanned<Expression>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token(); // `.`

        let start = self.left.span().start();

        Some(
            Expression::Property {
                left: Box::new(self.left),
                right: cursor.consume_identifier("property")?,
            }
            .at(Span::new(
                start,
                cursor.current.span().end(),
                cursor.file_id,
            )),
        )
    }
}

impl Parse for PrefixExpressionParser {
    type Output = Option<Spanned<Expression>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let operator_token = cursor.next.clone();
        let operator_span = operator_token.span();
        let operator: PrefixOperator = operator_token.unwrap().into();
        cursor.next_token();

        let inner = ExpressionParser {
            precedence: Precedence::Unary,
            ignore_struct: self.ignore_struct,
        }
        .parse_with(cursor)?;

        let span = Span::new(operator_span.start(), inner.span().end(), cursor.file_id);

        Some(
            Expression::Prefix {
                inner: Box::new(inner),
                operator: operator.at(operator_span),
            }
            .at(span),
        )
    }
}

impl Parse for PostfixExpressionParser {
    type Output = Option<Spanned<Expression>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let start = self.left.span().start();

        cursor.next_token();

        let span = Span::new(start, cursor.current.span().end(), cursor.file_id);
        let operator: PostfixOperator = cursor.current.unwrap().into();

        Some(
            Expression::Postfix {
                inner: Box::new(self.left),
                operator: operator.at(span),
            }
            .at(span),
        )
    }
}

impl Parse for ParenthesizedExpressionParser {
    type Output = Option<Spanned<Expression>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token();
        let start = cursor.current.span().start();

        let inner = ExpressionParser {
            precedence: Precedence::Lowest,
            ignore_struct: false,
        }
        .parse_with(cursor)?;

        cursor.consume(Token![')'], "parenthesized expression")?;

        Some(Expression::Parenthesized(Box::new(inner)).at(Span::new(
            start,
            cursor.current.span().end(),
            cursor.file_id,
        )))
    }
}

impl Parse for IfExpressionParser {
    type Output = Option<Spanned<Expression>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token(); // `if`

        let start = cursor.current.span().start();

        let mut condition_parser = ExpressionParser::default();
        condition_parser.ignore_struct();

        let condition = condition_parser.parse_with(cursor)?;
        let block = StatementsBlockParser.parse_with(cursor)?;

        let mut if_blocks = vec![(condition, block)];

        let mut r#else = None;

        while cursor.next.unwrap() == &Token![else] {
            cursor.next_token();

            if cursor.next.unwrap() != &Token![if] {
                r#else = Some(StatementsBlockParser.parse_with(cursor)?);
                break;
            }

            cursor.next_token();

            let mut condition_parser = ExpressionParser::default();
            condition_parser.ignore_struct();

            let condition = condition_parser.parse_with(cursor)?;
            let block = StatementsBlockParser.parse_with(cursor)?;

            if_blocks.push((condition, block));
        }

        Some(Expression::If { if_blocks, r#else }.at(Span::new(
            start,
            cursor.current.span().end(),
            cursor.file_id,
        )))
    }
}

impl Parse for CastExpressionParser {
    type Output = Option<Spanned<Expression>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let start = self.left.span().start();

        cursor.next_token();

        let right = TypeParser.parse_with(cursor)?;

        Some(
            Expression::As {
                left: Box::new(self.left),
                right,
            }
            .at(Span::new(
                start,
                cursor.current.span().end(),
                cursor.file_id,
            )),
        )
    }
}

impl Parse for CallExpressionParser {
    type Output = Option<Spanned<Expression>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let start = self.left.span().start();

        cursor.next_token(); // `(`

        let arguments = parse_list!(cursor, "call arguments list", Token![')'], || {
            ExpressionParser::default().parse_with(cursor)
        });

        cursor.next_token();

        Some(
            Expression::Call {
                left: Box::new(self.left),
                arguments,
            }
            .at(Span::new(
                start,
                cursor.current.span().end(),
                cursor.file_id,
            )),
        )
    }
}

impl Parse for BinaryExpressionParser {
    type Output = Option<Spanned<Expression>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let start = self.left.span().start();

        let operator_token = cursor.next.clone();
        let operator_span = operator_token.span();
        let operator: BinaryOperator = operator_token.unwrap().into();
        let precedence = cursor.next.unwrap().to_precedence();

        cursor.next_token();

        let right = ExpressionParser {
            precedence,
            ignore_struct: self.ignore_struct
        }.parse_with(cursor)?;

        Some(
            Expression::Binary {
                left: Box::new(self.left),
                right: Box::new(right),
                operator: operator.at(operator_span),
            }
            .at(Span::new(start, operator_span.end(), cursor.file_id)),
        )
    }
}

impl Parse for ArrayLiteralExpressionParser {
    type Output = Option<Spanned<Expression>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token();

        let start = cursor.next.span().start();

        let elements = parse_list!(cursor, "array literal", Token![']'], || {
            ExpressionParser::default().parse_with(cursor)
        });

        cursor.next_token();

        Some(Expression::Array { elements }.at(Span::new(
            start,
            cursor.current.span().end(),
            cursor.file_id,
        )))
    }
}

impl Parse for TupleExpressionParser {
    type Output = Option<Spanned<Expression>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token(); // `#`

        let start = cursor.current.span().start();

        cursor.consume(Token!['('], "tuple expression")?;

        let elements = parse_list!(cursor, "tuple expression", Token![')'], || {
            ExpressionParser::default().parse_with(cursor)
        });

        cursor.next_token(); // `)`

        Some(Expression::Tuple { elements }.at(Span::new(
            start,
            cursor.current.span().end(),
            cursor.file_id,
        )))
    }
}

impl Parse for StructExpressionParser {
    type Output = Option<Spanned<Expression>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token(); // `{`

        let fields = parse_list!(cursor, "struct expression", Token!['}'], || {
            StructExpressionUnitParser.parse_with(cursor)
        });

        cursor.next_token(); // `}`

        let span = Span::new(
            self.left.span().start(),
            cursor.current.span().end(),
            cursor.file_id,
        );

        Some(
            Expression::Struct {
                left: Box::new(self.left),
                fields,
            }
            .at(span),
        )
    }
}

impl Parse for StructExpressionUnitParser {
    type Output = Option<StructExpressionUnit>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let name = cursor.consume_identifier("struct field")?;

        let value = if cursor.next.unwrap() == &Token![:] {
            cursor.next_token();
            Some(ExpressionParser::default().parse_with(cursor)?)
        } else {
            None
        };

        Some(StructExpressionUnit { name, value })
    }
}

impl Parse for StatementsBlockExpressionParser {
    type Output = Option<Spanned<Expression>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        let start = cursor.next.span().start();

        let block = StatementsBlockParser.parse_with(cursor)?;
        let end = cursor.current.span().end();

        Some(Expression::StatementsBlock(block).at(Span::new(start, end, cursor.file_id)))
    }
}

impl Parse for FunctionExpressionParser {
    type Output = Option<Spanned<Expression>>;

    fn parse_with(self, cursor: &mut Cursor<'_>) -> Self::Output {
        cursor.next_token(); // `|`
        let start = cursor.current.span().start();

        let parameters = parse_list!(cursor, "function expression parameters", Token![|], || {
            FunctionParameterParser.parse_with(cursor)
        });

        cursor.next_token();

        let return_type = if cursor.next.unwrap() == &Token![:] {
            cursor.next_token();

            Some(TypeParser.parse_with(cursor)?)
        } else {
            None
        };

        let block = StatementsBlockParser.parse_with(cursor)?;

        Some(
            Expression::Function {
                parameters,
                return_type,
                block,
            }
            .at(Span::new(
                start,
                cursor.current.span().end(),
                cursor.file_id,
            )),
        )
    }
}
