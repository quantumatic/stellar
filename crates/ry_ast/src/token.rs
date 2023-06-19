//! `token.rs` - defines the token which represents grammatical unit of Ry
//! source text.
use crate::precedence::Precedence;
use phf::phf_map;
use ry_source_file::span::Span;
use std::fmt::Display;

/// Represents error that scanning process can fail with.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RawLexError {
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

impl AsRef<str> for RawLexError {
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

impl Display for RawLexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())?;
        Ok(())
    }
}

impl From<RawToken> for RawLexError {
    fn from(value: RawToken) -> Self {
        match value {
            RawToken::Error(e) => e,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct LexError {
    pub span: Span,
    pub raw: RawLexError,
}

/// Either the number is integer, float or imaginary literal.
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum NumberKind {
    Invalid,
    Int,
    Float,
}

/// This enum represents a set of keywords used in the Ry programming language.
/// Each variant of the enum corresponds to a specific keyword.
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
    Pub,
    Return,
    Struct,
    Trait,
    Type,
    Let,
    Where,
    While,
    Match,
    Use,
}

impl AsRef<str> for Keyword {
    fn as_ref(&self) -> &str {
        match self {
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
            Self::Let => "`let`",
            Self::Match => "`match`",
            Self::Use => "`use`",
        }
    }
}

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())?;
        Ok(())
    }
}

/// Represents a punctuator.
#[derive(Debug, Clone, PartialEq, Copy, Eq, Hash)]
pub enum Punctuator {
    /// Arrow (->).
    Arrow,

    /// Ampersand (&).
    And,

    /// Ampersand Equal (&=).
    AndEq,

    /// Logical And (&&).
    AndAnd,

    /// Assignment (=).
    Assign,

    /// Asterisk (*).
    Asterisk,

    /// Double Asterisk (**).
    AsteriskAsterisk,

    /// Asterisk Equal (*=).
    AsteriskEq,

    /// At Sign (@).
    AtSign,

    /// Bang (!).
    Bang,

    /// Close Brace (}).
    CloseBrace,

    /// Close Bracket (]).
    CloseBracket,

    /// Close Parenthesis ()).
    CloseParent,

    /// Colon (:).
    Colon,

    /// Comma (,).
    Comma,

    /// Dot (.).
    Dot,

    /// Dot Dot (..).
    DotDot,

    /// Equal (==).
    Eq,

    /// Greater Than (>).
    GreaterThan,

    /// Greater Than or Equal (>=).
    GreaterThanOrEq,

    /// Left Shift (<<).
    LeftShift,

    /// Less Than (<).
    LessThan,

    /// Less Than or Equal (<=).
    LessThanOrEq,

    /// Minus (-).
    Minus,

    /// Minus Equal (-=).
    MinusEq,

    /// Decrement (--).
    MinusMinus,

    /// Bitwise Not (~).
    Not,

    /// Not Equal (!=).
    NotEq,

    /// Open Brace ({).
    OpenBrace,

    /// Open Bracket ([).
    OpenBracket,

    /// Open Parenthesis (().
    OpenParent,

    /// Bitwise Or (|).
    Or,

    /// Or Equal (|=).
    OrEq,

    /// Logical Or (||).
    OrOr,

    /// Percent (%).
    Percent,

    /// Percent Equal (%=).
    PercentEq,

    /// Plus (+).
    Plus,

    /// Plus Equal (+=).
    PlusEq,

    /// Increment (++).
    PlusPlus,

    /// Question Mark (?).
    QuestionMark,

    /// Right Shift (>>).
    RightShift,

    /// Semicolon (;).
    Semicolon,

    /// Slash (/).
    Slash,

    /// Slash Equal (/=).
    SlashEq,

    /// Exclusive Or (^).
    Xor,

    /// Exclusive Or Equal (^=).
    XorEq,

    /// Hash Tag (#).
    HashTag,
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
            Self::Arrow => "`=>`",
            Self::NotEq => "`!=`",
            Self::RightShift => "`>>`",
            Self::LeftShift => "`<<`",
            Self::Or => "`|`",
            Self::And => "`&`",
            Self::AndEq => "`&=`",
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
            Self::DotDot => "`..`",
            Self::Semicolon => "`;`",
            Self::Colon => "`:`",
            Self::PlusPlus => "`++`",
            Self::MinusMinus => "`--`",
            Self::AsteriskAsterisk => "`**`",
            Self::Percent => "`%`",
            Self::PercentEq => "`%=`",
            Self::AtSign => "`@`",
            Self::HashTag => "`#`",
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
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub enum RawToken {
    /// True boolean literal (`true`).
    TrueBoolLiteral,
    /// False boolean literal (`false`).
    FalseBoolLiteral,
    /// Character literal.
    CharLiteral,
    /// Corresponds to any comment that is not a doc comment.
    Comment,
    /// Module level doc comment.
    GlobalDocComment,
    /// Item doc comment.
    LocalDocComment,
    /// End of file (`\0`).
    #[default]
    EndOfFile,
    /// Float literal.
    FloatLiteral,
    /// Identifier.
    Identifier,
    /// Integer literal.
    IntegerLiteral,
    /// Error token.
    Error(RawLexError),
    /// Keyword.
    Keyword(Keyword),
    /// Punctuator.
    Punctuator(Punctuator),
    /// String literal.
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
            Self::Identifier => "identifier",
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

impl From<RawLexError> for RawToken {
    fn from(value: RawLexError) -> Self {
        Self::Error(value)
    }
}

impl From<RawToken> for String {
    fn from(value: RawToken) -> Self {
        value.to_string()
    }
}

impl RawToken {
    pub fn eof(&self) -> bool {
        matches!(self, Self::EndOfFile)
    }
}

/// Represents a token with a specified location in source text.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Token {
    pub span: Span,
    pub raw: RawToken,
}

/// Macro used to easily initialize tokens.
///
/// # Example
/// ```
/// use ry_ast::{Token, token::{RawToken, Punctuator, Keyword}};
///
/// assert_eq!(Token![:], RawToken::Punctuator(Punctuator::Colon));
/// assert_eq!(Token![@], RawToken::Punctuator(Punctuator::AtSign));
///
/// // Same for keywords
/// assert_eq!(Token![use], RawToken::Keyword(Keyword::Use));
///
/// // For parenthesis and brackets use quotes.
/// assert_eq!(
///     Token!['('],
///     RawToken::Punctuator(Punctuator::OpenParent)
/// );
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
    [&=] =>                 {$crate::token::RawToken::Punctuator($crate::token::Punctuator::AndEq)};
    [&] =>                  {$crate::token::RawToken::Punctuator($crate::token::Punctuator::And)};
    [^=] =>                 {$crate::token::RawToken::Punctuator($crate::token::Punctuator::XorEq)};
    [^] =>                  {$crate::token::RawToken::Punctuator($crate::token::Punctuator::Xor)};
    [~] =>                  {$crate::token::RawToken::Punctuator($crate::token::Punctuator::Not)};
    ['('] =>                {$crate::token::RawToken::Punctuator($crate::token::Punctuator::OpenParent)};
    [')'] =>                {$crate::token::RawToken::Punctuator($crate::token::Punctuator::CloseParent)};
    ['['] =>                {$crate::token::RawToken::Punctuator($crate::token::Punctuator::OpenBracket)};
    [']'] =>                {$crate::token::RawToken::Punctuator($crate::token::Punctuator::CloseBracket)};
    ['{'] =>                {$crate::token::RawToken::Punctuator($crate::token::Punctuator::OpenBrace)};
    ['}'] =>                {$crate::token::RawToken::Punctuator($crate::token::Punctuator::CloseBrace)};
    [,] =>                  {$crate::token::RawToken::Punctuator($crate::token::Punctuator::Comma)};
    [.] =>                  {$crate::token::RawToken::Punctuator($crate::token::Punctuator::Dot)};
    [..] =>                 {$crate::token::RawToken::Punctuator($crate::token::Punctuator::DotDot)};
    [;] =>                  {$crate::token::RawToken::Punctuator($crate::token::Punctuator::Semicolon)};
    [%] =>                  {$crate::token::RawToken::Punctuator($crate::token::Punctuator::Percent)};
    [%=] =>                 {$crate::token::RawToken::Punctuator($crate::token::Punctuator::PercentEq)};
    [#] =>                  {$crate::token::RawToken::Punctuator($crate::token::Punctuator::HashTag)};
    [=>] =>                 {$crate::token::RawToken::Punctuator($crate::token::Punctuator::Arrow)};
    [true] =>               {$crate::token::RawToken::TrueBoolLiteral};
    [false] =>              {$crate::token::RawToken::FalseBoolLiteral};
    [use] =>                {$crate::token::RawToken::Keyword($crate::token::Keyword::Use)};
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
    [let] =>                {$crate::token::RawToken::Keyword($crate::token::Keyword::Let)};
    [as] =>                 {$crate::token::RawToken::Keyword($crate::token::Keyword::As)};
    [type] =>               {$crate::token::RawToken::Keyword($crate::token::Keyword::Type)};
    [for] =>                {$crate::token::RawToken::Keyword($crate::token::Keyword::For)};
    [where] =>              {$crate::token::RawToken::Keyword($crate::token::Keyword::Where)};
    [match] =>              {$crate::token::RawToken::Keyword($crate::token::Keyword::Match)};
}

/// List of reserved Ry names: keywords, boolean literals & etc..
pub static RESERVED: phf::Map<&'static str, RawToken> = phf_map! {
    "true" => RawToken::TrueBoolLiteral,
    "false" => RawToken::FalseBoolLiteral,
    "use" => Token![use],
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
    "let" => Token![let],
    "as" => Token![as],
    "type" => Token![type],
    "for" => Token![for],
    "where" => Token![where],
    "match" => Token![match],
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
            Self::OpenBracket => Precedence::GenericArgument,
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
            Self::OpenBrace => Precedence::Struct,
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

    #[inline]
    pub const fn binary_operator(&self) -> bool {
        matches!(
            self,
            Token![+=]
                | Token![+]
                | Token![-=]
                | Token![-]
                | Token![**]
                | Token![*=]
                | Token![*]
                | Token![/=]
                | Token![/]
                | Token![!=]
                | Token![!]
                | Token![>>]
                | Token![>=]
                | Token![>]
                | Token![<<]
                | Token![<=]
                | Token![<]
                | Token![==]
                | Token![=]
                | Token![|=]
                | Token![||]
                | Token![|]
                | Token![&]
                | Token![&&]
                | Token![&=]
                | Token![%]
                | Token![%=]
        )
    }

    #[inline]
    pub const fn prefix_operator(&self) -> bool {
        matches!(
            self,
            Token![!] | Token![~] | Token![++] | Token![--] | Token![-] | Token![+]
        )
    }

    #[inline]
    pub const fn postfix_operator(&self) -> bool {
        matches!(self, Token![?] | Token![++] | Token![--])
    }
}
