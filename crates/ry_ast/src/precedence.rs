//! Defines [`Precedence`] enum for different operator precedences.
//!
//! # Operator Precedence
//!
//! Operator precedence in a parser refers to the rules that determine the order
//! in which operators are evaluated in an expression. It specifies the hierarchy of
//! operators, ensuring that expressions are parsed correctly and unambiguously.
//!
//! See [`Precedence`] for more details.

/// Defines an enum representing different operator precedences.
///
/// In Ry programming language, operators have different levels of precedence.
/// Operators with higher precedence are evaluated before operators with lower
/// precedence. For example, in the expression `3 + 4 * 2`, the multiplication
/// operator (`*`) has higher precedence than the addition operator (`+`), so it is
/// evaluated first, resulting in `3 + (4 * 2) = 3 + 8 = 11`.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    /// Lowest precedence, corresponding to primary expressions: literals,
    /// if/else, match, etc.
    #[default]
    Lowest,

    /// Precedence corresponding to assignment operators: `=`, `+=`, `-=`,
    /// `*=`, `/=`, `^=`, `|=`.
    Assign,

    /// Precedence corresponding to binary expressions with `||` operator.
    OrOr,

    /// Precedence corresponding to binary expressions with `&&` operator.
    AndAnd,

    /// Precedence corresponding to binary expressions with `|` operator.
    Or,

    /// Precedence corresponding to binary expressions with `^` operator.
    Xor,

    /// Precedence corresponding to binary expressions with `&` operator.
    And,

    /// Precedence corresponding to binary expressions with equality operators:
    /// `==`, `!=`.
    Eq,

    /// Precedence corresponding to binary expressions with comparison operators:
    /// `<`, `<=`, `>`, `>=`.
    Comparison,

    /// Precedence corresponding to binary expressions with shift operators:
    /// `<<`, `>>`.
    LeftRightShift,

    /// Precedence corresponding to binary expressions with addition and
    /// subtraction operators: `+`, `-`.
    Sum,

    /// Precedence corresponding to binary expressions with multiplication and
    /// division operators: `*`, `/`.
    Product,

    /// Precedence corresponding to binary expressions with `**` operator.
    Power,

    /// Precedence corresponding to binary expressions with `%` operator.
    Mod,

    /// Precedence corresponding to cast expressions.
    As,

    /// Precedence corresponding to prefix or postfix expressions:
    ///
    /// ```txt
    /// a?
    /// !a
    /// a++
    /// --a
    /// ```
    Unary,

    /// Precedence corresponding to function calls.
    Call,

    /// Precedence corresponding to struct expressions.
    Struct,

    /// Precedence corresponding to property access expressions:
    /// ```txt
    /// a.b
    /// ```
    Property,

    /// Precedence corresponding to generic arguments expressions:
    /// ```txt
    /// a[i32]
    /// ```
    GenericArgument,
}
