//! `precedence.rs` - defines `Precedence` enum for different infix expression operator precedences.

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    #[default]
    Lowest,
    // ?:
    Elvis,
    // a = b | a += b | a -= b | a *= b | a /= b | a ^= b | a |= b
    Assign,
    // a || b
    OrOr,
    // a && b
    AndAnd,
    // a | b
    Or,
    // a ^ b
    Xor,
    // a & b
    And,
    // a == b | a != b
    Eq,
    // a > b | a < b | a >= b | a <= b
    LessOrGreater,
    // a >> b | a << b
    LeftRightShift,
    // a + b | a - b
    Sum,
    // a * b | a / b
    Product,
    // a ** b
    Power,
    // a % b
    Mod,
    // a as i32
    As,
    // !a | a?
    Unary,
    // a()
    Call,
    // a.b
    Property,
    // a[i32]
    TypeAnnotations,
}
