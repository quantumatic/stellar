//! `token.rs` - defines the token which represents grammatical unit of Ry
//! source text.

use crate::{precedence::Precedence, span::Spanned};
use phf::phf_map;
use ry_interner::Symbol;
use std::{fmt::Display, mem::discriminant, sync::Arc};

/// Represents error that lexer can fail with.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LexError {
    DigitDoesNotCorrespondToBase,
    EmptyCharLiteral,
    EmptyEscapeSequence,
    EmptyWrappedIdentifier,
    ExpectedCloseBracketInByteEscapeSequence,
    ExpectedCloseBracketInUnicodeEscapeSequence,
    ExpectedDigitInByteEscapeSequence,
    ExpectedDigitInUnicodeEscapeSequence,
    ExpectedOpenBracketInByteEscapeSequence,
    ExpectedOpenBracketInUnicodeEscapeSequence,
    ExponentHasNoDigits,
    ExponentRequiresDecimalMantissa,
    HasNoDigits,
    InvalidByteEscapeSequence,
    InvalidDigit,
    InvalidRadixPoint,
    InvalidUnicodeEscapeSequence,
    MoreThanOneCharInCharLiteral,
    NumberParseError,
    UnderscoreMustSeparateSuccessiveDigits,
    UnexpectedChar,
    UnknownEscapeSequence,
    UnterminatedCharLiteral,
    UnterminatedStringLiteral,
    UnterminatedWrappedIdentifier,
}

impl AsRef<str> for LexError {
    fn as_ref(&self) -> &str {
        match self {
            Self::EmptyCharLiteral => "empty character literal",
            Self::EmptyEscapeSequence => "empty escape sequence",
            Self::EmptyWrappedIdentifier => "empty wrapped identifier literal",
            Self::ExpectedCloseBracketInByteEscapeSequence => {
                "expected `}` in byte escape sequence"
            }
            Self::ExpectedCloseBracketInUnicodeEscapeSequence => {
                "expected `}` in Unicode escape sequence"
            }
            Self::ExpectedDigitInByteEscapeSequence => "expected digit in byte escape sequence",
            Self::ExpectedDigitInUnicodeEscapeSequence => {
                "expected digit in Unicode escape sequence"
            }
            Self::ExpectedOpenBracketInByteEscapeSequence => "expected `{` in byte escape sequence",
            Self::ExpectedOpenBracketInUnicodeEscapeSequence => {
                "expected `{` in Unicode escape sequence"
            }
            Self::ExponentHasNoDigits => "exponent has no digits",
            Self::ExponentRequiresDecimalMantissa => "exponent requires decimal mantissa",
            Self::DigitDoesNotCorrespondToBase => "digit doesn't correspond to the base",
            Self::HasNoDigits => "has no digits",
            Self::InvalidByteEscapeSequence => "invalid byte escape sequence",
            Self::InvalidDigit => "invalid digit",
            Self::InvalidRadixPoint => "invalid radix point",
            Self::InvalidUnicodeEscapeSequence => "invalid Unicode escape sequence",
            Self::MoreThanOneCharInCharLiteral => {
                "more than one character inside character literal"
            }
            Self::UnderscoreMustSeparateSuccessiveDigits => "`_` must separate successive digits",
            Self::NumberParseError => "number parsing error (overflow is possible)",
            Self::UnexpectedChar => "unexpected character",
            Self::UnknownEscapeSequence => "unknown escape sequence",
            Self::UnterminatedCharLiteral => "unterminated character literal",
            Self::UnterminatedStringLiteral => "unterminated string literal",
            Self::UnterminatedWrappedIdentifier => "unterminated wrapper identifier",
        }
    }
}

impl Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())?;
        Ok(())
    }
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
    As,
    Defer,
    Else,
    Enum,
    For,
    Fun,
    If,
    Impl,
    Import,
    Mut,
    Pub,
    Return,
    Struct,
    Trait,
    Var,
    Where,
    While,
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

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Punctuator {
    And,
    AndAnd,
    Assign,
    Asterisk,
    AsteriskAsterisk,
    AsteriskEq,
    AtSign,
    Bang,
    CloseBrace,
    CloseBracket,
    CloseParent,
    Colon,
    Comma,
    Dot,
    Elvis,
    Eq,
    GreaterThan,
    GreaterThanOrEq,
    LeftShift,
    LessThan,
    LessThanOrEq,
    Minus,
    MinusEq,
    MinusMinus,
    Not,
    NotEq,
    OpenBrace,
    OpenBracket,
    OpenParent,
    Or,
    OrEq,
    OrOr,
    Percent,
    Plus,
    PlusEq,
    PlusPlus,
    QuestionMark,
    RightShift,
    Semicolon,
    Slash,
    SlashEq,
    Xor,
    XorEq,
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
    BoolLiteral(bool),
    CharLiteral(char),

    /// Corresponds to any comment that is not a docstring.
    Comment,

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

    #[default]
    EndOfFile,

    FloatLiteral(f64),
    Identifier(Symbol),
    ImaginaryNumberLiteral(f64),
    IntegerLiteral(u64),

    Error(LexError),

    Keyword(Keyword),
    Punctuator(Punctuator),
    StringLiteral(Arc<str>),
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
            RawToken::Error(..) => "error token",
        }
    }
}

impl Display for RawToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())?;
        Ok(())
    }
}

impl From<RawToken> for String {
    fn from(value: RawToken) -> Self {
        value.to_string()
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
    pub fn to_precedence(&self) -> Precedence {
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
        }
    }
}

impl RawToken {
    pub fn to_precedence(&self) -> Precedence {
        match self {
            Self::Punctuator(punctuator) => punctuator.to_precedence(),
            Self::Keyword(Keyword::As) => Precedence::As,
            _ => unreachable!(),
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

#[test]
fn raw_token_size() {
    assert_eq!(std::mem::size_of::<RawToken>(), 24);
}
