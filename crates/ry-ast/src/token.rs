//! `token.rs` - defines the token which represents grammatical unit of Ry
//! source text.

use derive_more::Display;
use num_traits::ToPrimitive;
use phf::phf_map;
use std::mem::discriminant;
use string_interner::DefaultSymbol;
use thiserror::Error;

use crate::{location::WithSpan, precedence::Precedence};

/// Represents error that lexer can fail with.
#[derive(Error, Copy, Clone, Debug, PartialEq, Eq)]
pub enum LexerError {
    #[error("unexpected character `{_0}`")]
    UnexpectedChar(char),
    #[error("unterminated wrapped identifier literal")]
    UnterminatedWrappedIdentifierLiteral,
    #[error("empty wrapped identifier literal")]
    EmptyWrappedIdentifierLiteral,
    #[error("unterminated string literal")]
    UnterminatedStringLiteral,
    #[error("unknown escape sequence")]
    UnknownEscapeSequence,
    #[error("empty escape sequence")]
    EmptyEscapeSequence,
    #[error("expected closing bracket (`}}`) in unicode escape sequence")]
    ExpectedCloseBracketInUnicodeEscapeSequence,
    #[error("expected opening bracket (`{{`) in unicode escape sequence")]
    ExpectedOpenBracketInUnicodeEscapeSequence,
    #[error("expected hexadecimal digit in unicode escape sequence")]
    ExpectedDigitInUnicodeEscapeSequence,
    #[error("such unicode character does not exists")]
    InvalidUnicodeEscapeSequence,
    #[error("expected closing bracket (`}}`) in byte escape sequence")]
    ExpectedCloseBracketInByteEscapeSequence,
    #[error("expected opening bracket (`{{`) in byte escape sequence")]
    ExpectedOpenBracketInByteEscapeSequence,
    #[error("expected hexadecimal digit in byte escape sequence")]
    ExpectedDigitInByteEscapeSequence,
    #[error("such byte does not exists")]
    InvalidByteEscapeSequence,
    #[error("empty character literal")]
    EmptyCharLiteral,
    #[error("unterminated character literal")]
    UnterminatedCharLiteral,
    #[error("character literal can only one character long")]
    MoreThanOneCharInCharLiteral,
    #[error("invalid radix point")]
    InvalidRadixPoint,
    #[error("has no digits")]
    HasNoDigits,
    #[error("exponent requires decimal mantissa")]
    ExponentRequiresDecimalMantissa,
    #[error("exponent has no digits")]
    ExponentHasNoDigits,
    #[error("digit doesn't correspond to the base")]
    InvalidDigit,
    #[error("number parsing error (overflow is possible)")]
    NumberParserError,
    #[error("underscore must seperate successive digits")]
    UnderscoreMustSeperateSuccessiveDigits,
}

/// Wether the number is integer, float or imaginary literal.
#[derive(PartialEq, Debug)]
pub enum NumberKind {
    Invalid,
    Int,
    Float,
    Imag,
}

/// Represents token without a specific location in source text.
#[derive(Clone, Debug, PartialEq, Display, Default)]
pub enum RawToken {
    #[display(fmt = "identifier")]
    Identifier(DefaultSymbol),
    #[display(fmt = "string literal")]
    String(String),
    #[display(fmt = "integer literal")]
    Int(u64),
    #[display(fmt = "float literal")]
    Float(f64),
    #[display(fmt = "imaginary number literal")]
    Imag(f64),
    #[display(fmt = "character literal")]
    Char(char),
    #[display(fmt = "boolean literal")]
    Bool(bool),

    #[display(fmt = "`+`")]
    Plus,
    #[display(fmt = "`-`")]
    Minus,
    #[display(fmt = "`*`")]
    Asterisk,
    #[display(fmt = "`/`")]
    Slash,
    #[display(fmt = "`!`")]
    Bang,
    #[display(fmt = "`!!`")]
    BangBang,

    #[display(fmt = "`import`")]
    Import,
    #[display(fmt = "`pub`")]
    Pub,
    #[display(fmt = "`fun`")]
    Fun,
    #[display(fmt = "`struct`")]
    Struct,
    #[display(fmt = "`implement`")]
    Implement,
    #[display(fmt = "`trait`")]
    Trait,
    #[display(fmt = "`return`")]
    Return,
    #[display(fmt = "`defer`")]
    Defer,
    #[display(fmt = "`impl`")]
    Impl,
    #[display(fmt = "`impls`")]
    Impls,
    #[display(fmt = "`enum`")]
    Enum,
    #[display(fmt = "`if`")]
    If,
    #[display(fmt = "`else`")]
    Else,
    #[display(fmt = "`while`")]
    While,
    #[display(fmt = "`var`")]
    Var,
    #[display(fmt = "`as`")]
    As,
    #[display(fmt = "`for`")]
    For,
    #[display(fmt = "`mut`")]
    Mut,

    #[display(fmt = "`?`")]
    QuestionMark,

    #[display(fmt = "`>`")]
    GreaterThan,
    #[display(fmt = "`>=`")]
    GreaterThanOrEq,
    #[display(fmt = "`<`")]
    LessThan,
    #[display(fmt = "`<=`")]
    LessThanOrEq,
    #[display(fmt = "`=`")]
    Assign,
    #[display(fmt = "`==`")]
    Eq,
    #[display(fmt = "`!=`")]
    NotEq,

    #[display(fmt = "`>>`")]
    RightShift,
    #[display(fmt = "`<<`")]
    LeftShift,
    #[display(fmt = "`|`")]
    Or,
    #[display(fmt = "`&`")]
    And,
    #[display(fmt = "`^`")]
    Xor,
    #[display(fmt = "`~`")]
    Not,

    #[display(fmt = "`||`")]
    OrOr,
    #[display(fmt = "`&&`")]
    AndAnd,

    #[display(fmt = "`$`")]
    Dollar,

    #[display(fmt = "`+=`")]
    PlusEq,
    #[display(fmt = "`-=`")]
    MinusEq,
    #[display(fmt = "`*=`")]
    AsteriskEq,
    #[display(fmt = "`/=`")]
    SlashEq,
    #[display(fmt = "`^=`")]
    XorEq,
    #[display(fmt = "`|=`")]
    OrEq,

    #[display(fmt = "`(`")]
    OpenParent,
    #[display(fmt = "`)`")]
    CloseParent,
    #[display(fmt = "`[`")]
    OpenBracket,
    #[display(fmt = "`]`")]
    CloseBracket,
    #[display(fmt = "`{{`")]
    OpenBrace,
    #[display(fmt = "`}}`")]
    CloseBrace,

    #[display(fmt = "`,`")]
    Comma,
    #[display(fmt = "`.`")]
    Dot,
    #[display(fmt = "`;`")]
    Semicolon,
    #[display(fmt = "`:`")]
    Colon,
    #[display(fmt = "`::`")]
    DoubleColon,

    #[display(fmt = "`++`")]
    PlusPlus,
    #[display(fmt = "`--`")]
    MinusMinus,
    #[display(fmt = "`**`")]
    AsteriskAsterisk,

    #[display(fmt = "`%`")]
    Percent,
    #[display(fmt = "`?:`")]
    Elvis,

    #[display(fmt = "`@`")]
    AtSign,

    #[display(fmt = "comment")]
    Comment(String),

    #[default]
    #[display(fmt = "end of file")]
    EndOfFile,

    #[display(fmt = "invalid token")]
    Invalid(LexerError),
}

impl AsRef<RawToken> for RawToken {
    fn as_ref(&self) -> &Self {
        self
    }
}

pub type Token = WithSpan<RawToken>;

/// List of reserved Ry names: keywords, boolean literals & etc..
pub static RESERVED: phf::Map<&'static str, RawToken> = phf_map! {
    "true" => RawToken::Bool(true),
    "false" => RawToken::Bool(false),
    "import" => RawToken::Import,
    "pub" => RawToken::Pub,
    "fun" => RawToken::Fun,
    "struct" => RawToken::Struct,
    "implement" => RawToken::Implement,
    "trait" => RawToken::Trait,
    "return" => RawToken::Return,
    "defer" => RawToken::Defer,
    "impl" => RawToken::Impl,
    "enum" => RawToken::Enum,
    "if" => RawToken::If,
    "else" => RawToken::Else,
    "while" => RawToken::While,
    "var" => RawToken::Var,
    "as" => RawToken::As,
    "for" => RawToken::For,
    "mut" => RawToken::Mut,
};

impl RawToken {
    pub fn to_precedence(&self) -> i8 {
        match self {
            Self::Elvis => Precedence::Elvis,
            Self::OrOr => Precedence::OrOr,
            Self::AndAnd => Precedence::AndAnd,
            Self::Or => Precedence::Or,
            Self::Xor => Precedence::Xor,
            Self::And => Precedence::And,
            Self::Eq | Self::NotEq => Precedence::Eq,
            Self::Assign
            | Self::PlusEq
            | Self::MinusEq
            | Self::AsteriskEq
            | Self::SlashEq
            | Self::OrEq
            | Self::XorEq => Precedence::Assign,
            Self::LessThan | Self::LessThanOrEq | Self::GreaterThan | Self::GreaterThanOrEq => {
                Precedence::LessOrGreater
            }
            Self::Dollar => Precedence::Dollar,
            Self::LeftShift | Self::RightShift => Precedence::LeftRightShift,
            Self::Plus | Self::Minus => Precedence::Sum,
            Self::Asterisk | Self::Slash => Precedence::Product,
            Self::AsteriskAsterisk => Precedence::Power,
            Self::Percent => Precedence::Mod,
            Self::OpenParent => Precedence::Call,
            Self::OpenBracket | Self::Dot => Precedence::Index,
            Self::Not
            | Self::PlusPlus
            | Self::MinusMinus
            | Self::Bang
            | Self::QuestionMark
            | Self::BangBang => Precedence::PrefixOrPostfix,
            Self::As => Precedence::As,
            _ => Precedence::Lowest,
        }
        .to_i8()
        .unwrap()
    }

    pub fn is<T: AsRef<Self>>(&self, raw: T) -> bool {
        discriminant(self) == discriminant(raw.as_ref())
    }

    pub fn is_one_of<T: AsRef<Self>>(&self, raws: &[T]) -> bool {
        for raw in raws {
            if discriminant(self) == discriminant(raw.as_ref()) {
                return true;
            }
        }

        false
    }
}
