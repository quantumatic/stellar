//! `token.rs` - defines the token which represents grammatical unit of Ry
//! source text.
use crate::{precedence::Precedence, span::Spanned};
use phf::phf_map;
use ry_interner::Symbol;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Represents error that lexer can fail with.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
}

#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]
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
    Pub,
    Return,
    Struct,
    Trait,
    Type,
    Var,
    Where,
    While,
}

impl AsRef<str> for Keyword {
    fn as_ref(&self) -> &str {
        match self {
            Self::Import => "`import`",
            Self::Pub => "`pub`",
            Self::Fun => "`fun`",
            Self::Struct => "`struct`",
            Self::Trait => "`trait`",
            Self::Type => "`type`",
            Self::Return => "`return`",
            Self::Defer => "`defer`",
            Self::Impl => "`impl`",
            Self::Enum => "`enum`",
            Self::If => "`if`",
            Self::Else => "`else`",
            Self::While => "`while`",
            Self::As => "`as`",
            Self::For => "`for`",
            Self::Where => "`where`",
            Self::Var => "`var`",
        }
    }
}

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]
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
            Self::Plus => "`+`",
            Self::Minus => "`-`",
            Self::Asterisk => "`*`",
            Self::Slash => "`/`",
            Self::Bang => "`!`",
            Self::QuestionMark => "`?`",
            Self::GreaterThan => "`>`",
            Self::GreaterThanOrEq => "`>=`",
            Self::LessThan => "`<`",
            Self::LessThanOrEq => "`<=`",
            Self::Assign => "`=`",
            Self::Eq => "`==`",
            Self::NotEq => "`!=`",
            Self::RightShift => "`>>`",
            Self::LeftShift => "`<<`",
            Self::Or => "`|`",
            Self::And => "`&`",
            Self::Xor => "`^`",
            Self::Not => "`~`",
            Self::OrOr => "`||`",
            Self::AndAnd => "`&&`",
            Self::PlusEq => "`+=`",
            Self::MinusEq => "`-=`",
            Self::AsteriskEq => "`*=`",
            Self::SlashEq => "`/=`",
            Self::XorEq => "`^=`",
            Self::OrEq => "`|=`",
            Self::OpenParent => "`(`",
            Self::CloseParent => "`)`",
            Self::OpenBracket => "`[`",
            Self::CloseBracket => "`]`",
            Self::OpenBrace => "`{`",
            Self::CloseBrace => "`}`",
            Self::Comma => "`,`",
            Self::Dot => "`.`",
            Self::Semicolon => "`;`",
            Self::Colon => "`:`",
            Self::PlusPlus => "`++`",
            Self::MinusMinus => "`--`",
            Self::AsteriskAsterisk => "`**`",
            Self::Percent => "`%`",
            Self::AtSign => "`@`",
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
#[derive(Copy, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub enum RawToken {
    TrueBoolLiteral,
    FalseBoolLiteral,

    CharLiteral,

    /// Corresponds to any comment that is not a doc comment.
    Comment,

    GlobalDocComment,
    LocalDocComment,

    #[default]
    EndOfFile,

    FloatLiteral,
    Identifier(Symbol),
    IntegerLiteral,

    Error(LexError),

    Keyword(Keyword),
    Punctuator(Punctuator),
    StringLiteral,
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
            Self::StringLiteral => "string literal",
            Self::IntegerLiteral => "integer literal",
            Self::FloatLiteral => "float literal",
            Self::CharLiteral => "character literal",
            Self::TrueBoolLiteral => "`true`",
            Self::FalseBoolLiteral => "`false`",
            Self::Keyword(keyword) => keyword.as_ref(),
            Self::Punctuator(punctuator) => punctuator.as_ref(),
            Self::GlobalDocComment | Self::LocalDocComment => "doc comment",
            Self::Comment => "comment",
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

#[macro_export]
macro_rules! Token {
    [:] =>                  {$crate::token::RawToken::Punctuator($crate::token::Punctuator::Colon)};
    [@] =>                  {$crate::token::RawToken::Punctuator($crate::token::Punctuator::AtSign)};
    [++] =>                 {$crate::token::RawToken::Punctuator($crate::token::Punctuator::PlusPlus)};
    [+=] =>                 {$crate::token::RawToken::Punctuator($crate::token::Punctuator::PlusEq)};
    [+] =>                  {$crate::token::RawToken::Punctuator($crate::token::Punctuator::Plus)};
    [--] =>                 {$crate::token::RawToken::Punctuator($crate::token::Punctuator::MinusMinus)};
    [-=] =>                 {$crate::token::RawToken::Punctuator($crate::token::Punctuator::MinusEq)};
    [-] =>                  {$crate::token::RawToken::Punctuator($crate::token::Punctuator::Minus)};
    [**] =>                 {$crate::token::RawToken::Punctuator($crate::token::Punctuator::AsteriskAsterisk)};
    [*=] =>                 {$crate::token::RawToken::Punctuator($crate::token::Punctuator::AsteriskEq)};
    [*] =>                  {$crate::token::RawToken::Punctuator($crate::token::Punctuator::Asterisk)};
    [/=] =>                 {$crate::token::RawToken::Punctuator($crate::token::Punctuator::SlashEq)};
    [/] =>                  {$crate::token::RawToken::Punctuator($crate::token::Punctuator::Slash)};
    [!=] =>                 {$crate::token::RawToken::Punctuator($crate::token::Punctuator::NotEq)};
    [!] =>                  {$crate::token::RawToken::Punctuator($crate::token::Punctuator::Bang)};
    [>>] =>                 {$crate::token::RawToken::Punctuator($crate::token::Punctuator::RightShift)};
    [>=] =>                 {$crate::token::RawToken::Punctuator($crate::token::Punctuator::GreaterThanOrEq)};
    [>] =>                  {$crate::token::RawToken::Punctuator($crate::token::Punctuator::GreaterThan)};
    [<<] =>                 {$crate::token::RawToken::Punctuator($crate::token::Punctuator::LeftShift)};
    [<=] =>                 {$crate::token::RawToken::Punctuator($crate::token::Punctuator::LessThanOrEq)};
    [<] =>                  {$crate::token::RawToken::Punctuator($crate::token::Punctuator::LessThan)};
    [==] =>                 {$crate::token::RawToken::Punctuator($crate::token::Punctuator::Eq)};
    [=] =>                  {$crate::token::RawToken::Punctuator($crate::token::Punctuator::Assign)};
    [|=] =>                 {$crate::token::RawToken::Punctuator($crate::token::Punctuator::OrEq)};
    [||] =>                 {$crate::token::RawToken::Punctuator($crate::token::Punctuator::OrOr)};
    [|] =>                  {$crate::token::RawToken::Punctuator($crate::token::Punctuator::Or)};
    [?] =>                  {$crate::token::RawToken::Punctuator($crate::token::Punctuator::QuestionMark)};
    [&&] =>                 {$crate::token::RawToken::Punctuator($crate::token::Punctuator::AndAnd)};
    [&] =>                  {$crate::token::RawToken::Punctuator($crate::token::Punctuator::And)};
    [^=] =>                 {$crate::token::RawToken::Punctuator($crate::token::Punctuator::XorEq)};
    [^] =>                  {$crate::token::RawToken::Punctuator($crate::token::Punctuator::Xor)};
    [~=] =>                 {$crate::token::RawToken::Punctuator($crate::token::Punctuator::NotEq)};
    [~] =>                  {$crate::token::RawToken::Punctuator($crate::token::Punctuator::Not)};
    ['('] =>                {$crate::token::RawToken::Punctuator($crate::token::Punctuator::OpenParent)};
    [')'] =>                {$crate::token::RawToken::Punctuator($crate::token::Punctuator::CloseParent)};
    ['['] =>                {$crate::token::RawToken::Punctuator($crate::token::Punctuator::OpenBracket)};
    [']'] =>                {$crate::token::RawToken::Punctuator($crate::token::Punctuator::CloseBracket)};
    ['{'] =>                {$crate::token::RawToken::Punctuator($crate::token::Punctuator::OpenBrace)};
    ['}'] =>                {$crate::token::RawToken::Punctuator($crate::token::Punctuator::CloseBrace)};
    [,] =>                  {$crate::token::RawToken::Punctuator($crate::token::Punctuator::Comma)};
    [.] =>                  {$crate::token::RawToken::Punctuator($crate::token::Punctuator::Dot)};
    [;] =>                  {$crate::token::RawToken::Punctuator($crate::token::Punctuator::Semicolon)};
    [%] =>                  {$crate::token::RawToken::Punctuator($crate::token::Punctuator::Percent)};
    [true] =>               {$crate::token::RawToken::TrueBoolLiteral};
    [false] =>              {$crate::token::RawToken::FalseBoolLiteral};
    [import] =>             {$crate::token::RawToken::Keyword($crate::token::Keyword::Import)};
    [pub] =>                {$crate::token::RawToken::Keyword($crate::token::Keyword::Pub)};
    [fun] =>                {$crate::token::RawToken::Keyword($crate::token::Keyword::Fun)};
    [struct] =>             {$crate::token::RawToken::Keyword($crate::token::Keyword::Struct)};
    [trait] =>              {$crate::token::RawToken::Keyword($crate::token::Keyword::Trait)};
    [return] =>             {$crate::token::RawToken::Keyword($crate::token::Keyword::Return)};
    [defer] =>              {$crate::token::RawToken::Keyword($crate::token::Keyword::Defer)};
    [impl] =>               {$crate::token::RawToken::Keyword($crate::token::Keyword::Impl)};
    [enum] =>               {$crate::token::RawToken::Keyword($crate::token::Keyword::Enum)};
    [if] =>                 {$crate::token::RawToken::Keyword($crate::token::Keyword::If)};
    [else] =>               {$crate::token::RawToken::Keyword($crate::token::Keyword::Else)};
    [while] =>              {$crate::token::RawToken::Keyword($crate::token::Keyword::While)};
    [var] =>                {$crate::token::RawToken::Keyword($crate::token::Keyword::Var)};
    [as] =>                 {$crate::token::RawToken::Keyword($crate::token::Keyword::As)};
    [type] =>               {$crate::token::RawToken::Keyword($crate::token::Keyword::Type)};
    [for] =>                {$crate::token::RawToken::Keyword($crate::token::Keyword::For)};
    [where] =>              {$crate::token::RawToken::Keyword($crate::token::Keyword::Where)};
}

/// List of reserved Ry names: keywords, boolean literals & etc..
pub static RESERVED: phf::Map<&'static str, RawToken> = phf_map! {
    "true" => RawToken::TrueBoolLiteral,
    "false" => RawToken::FalseBoolLiteral,
    "import" => Token![import],
    "pub" => Token![pub],
    "fun" => Token![fun],
    "struct" => Token![struct],
    "trait" => Token![trait],
    "return" => Token![return],
    "defer" => Token![defer],
    "impl" => Token![impl],
    "enum" => Token![enum],
    "if" => Token![if],
    "else" => Token![else],
    "while" => Token![while],
    "var" => Token![var],
    "as" => Token![as],
    "type" => Token![type],
    "for" => Token![for],
    "where" => Token![where]
};

impl Punctuator {
    pub fn to_precedence(&self) -> Precedence {
        match self {
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
            _ => Precedence::Lowest,
        }
    }
}

#[test]
fn raw_token_size() {
    assert_eq!(std::mem::size_of::<RawToken>(), 16);
}
