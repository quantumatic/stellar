//! `token.rs` - defines the token which represents grammatical unit of Ry
//! source text.

use phf::phf_map;
use ry_interner::Symbol;
use std::{fmt::Display, mem::discriminant, sync::Arc};
use thiserror::Error;

use crate::{precedence::Precedence, span::Spanned};

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
    #[error("underscore must separate successive digits")]
    UnderscoreMustSeparateSuccessiveDigits,
}

/// Either the number is integer, float or imaginary literal.
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum NumberKind {
    Invalid,
    Int,
    Float,
    Imag,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Keyword {
    Import,
    Pub,
    Fun,
    Struct,
    Trait,
    Return,
    Defer,
    Impl,
    Enum,
    If,
    Else,
    While,
    As,
    For,
    Mut,
    Where,
    Var,
}

impl AsRef<str> for Keyword {
    fn as_ref(&self) -> &str {
        match self {
            Keyword::Import => "import",
            Keyword::Pub => "pub",
            Keyword::Fun => "fun",
            Keyword::Struct => "struct",
            Keyword::Trait => "trait",
            Keyword::Return => "return",
            Keyword::Defer => "defer",
            Keyword::Impl => "impl",
            Keyword::Enum => "enum",
            Keyword::If => "if",
            Keyword::Else => "else",
            Keyword::While => "while",
            Keyword::As => "as",
            Keyword::For => "for",
            Keyword::Mut => "mut",
            Keyword::Where => "where",
            Keyword::Var => "var",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Punctuator {
    Plus,
    Minus,
    Asterisk,
    Slash,
    Bang,
    QuestionMark,
    GreaterThan,
    GreaterThanOrEq,
    LessThan,
    LessThanOrEq,
    Assign,
    Eq,
    NotEq,
    RightShift,
    LeftShift,
    Or,
    And,
    Xor,
    Not,
    OrOr,
    AndAnd,
    PlusEq,
    MinusEq,
    AsteriskEq,
    SlashEq,
    XorEq,
    OrEq,
    OpenParent,
    CloseParent,
    OpenBracket,
    CloseBracket,
    OpenBrace,
    CloseBrace,
    Comma,
    Dot,
    Semicolon,
    Colon,
    PlusPlus,
    MinusMinus,
    AsteriskAsterisk,
    Percent,
    Elvis,
    AtSign,
}

impl AsRef<str> for Punctuator {
    fn as_ref(&self) -> &str {
        match self {
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Asterisk => "*",
            Self::Slash => "/",
            Self::Bang => "!",
            Self::QuestionMark => "?",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEq => ">=",
            Self::LessThan => "<",
            Self::LessThanOrEq => "<=",
            Self::Assign => "=",
            Self::Eq => "==",
            Self::NotEq => "!=",
            Self::RightShift => ">>",
            Self::LeftShift => "<<",
            Self::Or => "|",
            Self::And => "&",
            Self::Xor => "^",
            Self::Not => "~",
            Self::OrOr => "||",
            Self::AndAnd => "&&",
            Self::PlusEq => "+=",
            Self::MinusEq => "-=",
            Self::AsteriskEq => "*=",
            Self::SlashEq => "/=",
            Self::XorEq => "^=",
            Self::OrEq => "|=",
            Self::OpenParent => "(",
            Self::CloseParent => ")",
            Self::OpenBracket => "[",
            Self::CloseBracket => "]",
            Self::OpenBrace => "{",
            Self::CloseBrace => "}",
            Self::Comma => ",",
            Self::Dot => ".",
            Self::Semicolon => ";",
            Self::Colon => ":",
            Self::PlusPlus => "++",
            Self::MinusMinus => "--",
            Self::AsteriskAsterisk => "**",
            Self::Percent => "%",
            Self::Elvis => "?:",
            Self::AtSign => "@",
        }
    }
}

impl Display for Punctuator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())?;

        Ok(())
    }
}

/// Represents token without a specific location in source text.
#[derive(Clone, Debug, PartialEq, Default)]
pub enum RawToken {
    Identifier(Symbol),
    StringLiteral(Arc<str>),
    IntegerLiteral(u64),
    FloatLiteral(f64),
    ImaginaryNumberLiteral(f64),
    CharLiteral(char),
    BoolLiteral(bool),
    Punctuator(Punctuator),
    Keyword(Keyword),
    /// [`global`] here is either the docstring is declared for the whole module
    /// or only for a given item, enum variant or trait method.
    ///
    /// [`content`] corresponds to the contents of the docstring but
    /// without first 3 characters which are:
    /// - two initial slashes - `//`
    /// - last character - either `/` or `!`
    DocstringComment {
        global: bool,
        content: Arc<str>,
    },
    /// Corresponds to any comment that is not a docstring.
    Comment,
    #[default]
    EndOfFile,
    Invalid(LexerError),
}

impl AsRef<RawToken> for RawToken {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsRef<str> for RawToken {
    fn as_ref(&self) -> &str {
        match self {
            Self::Identifier(..) => "identifier",
            Self::StringLiteral(..) => "string literal",
            Self::IntegerLiteral(..) => "integer literal",
            Self::FloatLiteral(..) => "float literal",
            Self::ImaginaryNumberLiteral(..) => "imaginary number literal",
            Self::CharLiteral(..) => "character literal",
            Self::BoolLiteral(..) => "bool literal",
            Self::Keyword(keyword) => keyword.as_ref(),
            Self::Punctuator(punctuator) => punctuator.as_ref(),
            Self::DocstringComment { .. } | Self::Comment => "comment",
            Self::EndOfFile => "end of file",
            RawToken::Invalid(..) => "invalid token",
        }
    }
}

impl Display for RawToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())?;

        Ok(())
    }
}

pub type Token = Spanned<RawToken>;

/// List of reserved Ry names: keywords, boolean literals & etc..
pub static RESERVED: phf::Map<&'static str, RawToken> = phf_map! {
    "true" => RawToken::BoolLiteral(true),
    "false" => RawToken::BoolLiteral(false),
    "import" => RawToken::Keyword(Keyword::Import),
    "pub" => RawToken::Keyword(Keyword::Pub),
    "fun" => RawToken::Keyword(Keyword::Fun),
    "struct" => RawToken::Keyword(Keyword::Struct),
    "trait" => RawToken::Keyword(Keyword::Trait),
    "return" => RawToken::Keyword(Keyword::Return),
    "defer" => RawToken::Keyword(Keyword::Defer),
    "impl" => RawToken::Keyword(Keyword::Impl),
    "enum" => RawToken::Keyword(Keyword::Enum),
    "if" => RawToken::Keyword(Keyword::If),
    "else" => RawToken::Keyword(Keyword::Else),
    "while" => RawToken::Keyword(Keyword::While),
    "var" => RawToken::Keyword(Keyword::Var),
    "as" => RawToken::Keyword(Keyword::As),
    "for" => RawToken::Keyword(Keyword::For),
    "mut" => RawToken::Keyword(Keyword::Mut),
    "where" => RawToken::Keyword(Keyword::Where)
};

impl Punctuator {
    pub fn to_precedence(&self) -> i8 {
        (match self {
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
            Self::OpenBracket => Precedence::TypeAnnotations,
            Self::LeftShift | Self::RightShift => Precedence::LeftRightShift,
            Self::Plus | Self::Minus => Precedence::Sum,
            Self::Asterisk | Self::Slash => Precedence::Product,
            Self::AsteriskAsterisk => Precedence::Power,
            Self::Percent => Precedence::Mod,
            Self::OpenParent => Precedence::Call,
            Self::Dot => Precedence::Property,
            Self::Not | Self::PlusPlus | Self::MinusMinus | Self::Bang | Self::QuestionMark => {
                Precedence::Unary
            }
            _ => Precedence::Lowest,
        }) as i8
    }
}

impl RawToken {
    pub fn to_precedence(&self) -> i8 {
        match self {
            Self::Punctuator(punctuator) => punctuator.to_precedence(),
            Self::Keyword(Keyword::As) => Precedence::As as i8,
            _ => unreachable!(),
        }
    }

    pub fn is<T: AsRef<Self>>(&self, raw: T) -> bool {
        if let Self::Punctuator(p) = self {
            if let Self::Punctuator(p2) = raw.as_ref() {
                discriminant(p) == discriminant(p2)
            } else {
                false
            }
        } else if let Self::Keyword(k) = self {
            if let Self::Keyword(k2) = raw.as_ref() {
                discriminant(k) == discriminant(k2)
            } else {
                false
            }
        } else {
            discriminant(self) == discriminant(raw.as_ref())
        }
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
