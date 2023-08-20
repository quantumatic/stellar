//! Defines a [`Token`] which represents grammatical unit of Ry source text.

use derive_more::Display;
use ry_filesystem::location::Location;

use crate::precedence::Precedence;

/// Represents error that scanning process can fail with.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Display)]
pub enum RawLexError {
    DigitDoesNotCorrespondToBase,
    #[display(fmt = "empty character literal")]
    EmptyCharacterLiteral,
    #[display(fmt = "empty escape sequence")]
    EmptyEscapeSequence,
    #[display(fmt = "empty wrapped identifier literal")]
    EmptyWrappedIdentifier,
    #[display(fmt = "expected `}}` in byte escape sequence")]
    ExpectedCloseBracketInByteEscapeSequence,
    #[display(fmt = "expected `}}` in Unicode escape sequence")]
    ExpectedCloseBracketInUnicodeEscapeSequence,
    #[display(fmt = "expected digit in byte escape sequence")]
    ExpectedDigitInByteEscapeSequence,
    #[display(fmt = "expected digit in Unicode escape sequence")]
    ExpectedDigitInUnicodeEscapeSequence,
    #[display(fmt = "expected `{{` in byte escape sequence")]
    ExpectedOpenBracketInByteEscapeSequence,
    #[display(fmt = "expected `{{` in Unicode escape sequence")]
    ExpectedOpenBracketInUnicodeEscapeSequence,
    #[display(fmt = "exponent has no digits")]
    ExponentHasNoDigits,
    #[display(fmt = "exponent requires decimal mantissa")]
    ExponentRequiresDecimalMantissa,
    #[display(fmt = "number contains no digits")]
    NumberContainsNoDigits,
    #[display(fmt = "invalid byte escape sequence")]
    InvalidByteEscapeSequence,
    #[display(fmt = "invalid digit")]
    InvalidDigit,
    #[display(fmt = "invalid radix point")]
    InvalidRadixPoint,
    #[display(fmt = "invalid Unicode escape sequence")]
    InvalidUnicodeEscapeSequence,
    #[display(fmt = "more than one character in character literal")]
    MoreThanOneCharInCharLiteral,
    #[display(fmt = "number cannot be parsed")]
    NumberParseError,
    #[display(fmt = "underscore must separate successive digits")]
    UnderscoreMustSeparateSuccessiveDigits,
    #[display(fmt = "unexpected character")]
    UnexpectedChar,
    #[display(fmt = "unknown escape sequence")]
    UnknownEscapeSequence,
    #[display(fmt = "untermined character literal")]
    UnterminatedCharLiteral,
    #[display(fmt = "unterminated string literal")]
    UnterminatedStringLiteral,
    #[display(fmt = "unterminated wrapped identifier")]
    UnterminatedWrappedIdentifier,
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
    pub location: Location,
    pub raw: RawLexError,
}

/// Either the number is integer or float.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NumberKind {
    Invalid,
    Int,
    Float,
}

macro_rules! define_keywords {
    {$($value:literal => $keyword:ident),*} => {
        /// This enum represents a set of keywords used in the Ry programming language.
        /// Each variant of the enum corresponds to a specific keyword.
        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        pub enum Keyword {
            $(
                #[doc = concat!("Keyword `", $value, "`.")]
                $keyword,
            )*
        }

        use std::fmt::Display;

        impl Display for Keyword {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$keyword => write!(f, "`{}`", $value),
                    )*
                }
            }
        }

        /// Convert a string into a keyword.
        pub fn get_keyword(string: impl AsRef<str>) -> Option<Keyword> {
            match string.as_ref() {
                $(
                    $value => Some(Keyword::$keyword),
                )*
                _ => None,
            }
        }
    };
}

macro_rules! define_punctuators {
    ($(
        $(#[$($doc:tt)*])*
        $punctuator:ident => $value:literal
    ),*) => {
        /// Represents a punctuator.
        #[derive(Debug, Clone, PartialEq, Copy, Eq, Hash)]
        pub enum Punctuator {
            $(
                $(#[$($doc)*])*
                $punctuator,
            )*
        }

        impl Display for Punctuator {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$punctuator => write!(f, "`{}`", $value),
                    )*
                }
            }
        }

        impl From<&str> for Punctuator {
            fn from(value: &str) -> Self {
                match value {
                    $(
                        $value => Self::$punctuator,
                    )*
                    _ => unreachable!(),
                }
            }
        }
    };
}

define_keywords! {
    "as" => As, "defer" => Defer, "else" => Else,
    "enum" => Enum, "for" => For, "fun" => Fun,
    "if" => If, "pub" => Pub, "return" => Return,
    "struct" => Struct, "type" => Type, "let" => Let,
    "where" => Where, "while" => While, "match" => Match,
    "import" => Import, "break" => Break, "continue" => Continue,
    "dyn" => Dyn, "loop" => Loop, "interface" => Interface,
    "implements" => Implements
}

define_punctuators! {
    /// Arrow (`->`).
    Arrow => "->",

    /// Ampersand (`&`).
    Ampersand => "&",

    /// Ampersand Equal (`&=`).
    AmpersandEq => "&=",

    /// Double Ampersand (`&&`).
    DoubleAmpersand => "&&",

    /// Asterisk (`*`).
    Asterisk => "*",

    /// Double Asterisk (`**`).
    DoubleAsterisk => "**",

    /// Asterisk Equal (`*=`).
    AsteriskEq => "*=",

    /// At Sign (`@`).
    At => "@",

    /// Bang (`!`).
    Bang => "!",

    /// Close Brace (`}`).
    CloseBrace => "}",

    /// Close Bracket (`]`).
    CloseBracket => "]",

    /// Close Parenthesis (`)`).
    CloseParent => ")",

    /// Colon (`:`).
    Colon => ":",

    /// Comma (`,`).
    Comma => ",",

    /// Dot (`.`).
    Dot => ".",

    /// Dot Dot (`..`).
    DoubleDot => "..",

    /// Equal (`=`).
    Eq => "=",

    /// Double Equal (`==`).
    DoubleEq => "==",

    /// Greater (`>`).
    Greater => ">",

    /// Greater Or Equal (`>=`).
    GreaterEq => ">=",

    /// Left Shift (`<<`).
    LeftShift => "<<",

    /// Less (`<`).
    Less => "<",

    /// Less or Equal (`<=`).
    LessEq => "<=",

    /// Minus (`-`).
    Minus => "-",

    /// Minus Equal (`-=`).
    MinusEq => "-=",

    /// Double Minus (`--`).
    DoubleMinus => "--",

    /// Tilde (`~`).
    Tilde => "~",

    /// Bang Equal (`!=`).
    BangEq => "!=",

    /// Open Brace (`{`).
    OpenBrace => "{",

    /// Open Bracket (`[`).
    OpenBracket => "[",

    /// Open Parenthesis (`(`).
    OpenParent => "(",

    /// Or (`|`).
    Or => "|",

    /// Or Equal (`|=`).
    OrEq => "|=",

    /// Logical Or (`||`).
    DoubleOr => "||",

    /// Percent (`%`).
    Percent => "%",

    /// Percent Equal (`%=`).
    PercentEq => "%=",

    /// Plus (`+`).
    Plus => "+",

    /// Plus Equal (`+=`).
    PlusEq => "+=",

    /// Double Plus (`++`).
    DoublePlus => "++",

    /// Question Mark (`?`).
    QuestionMark => "?",

    /// Right Shift (`>>`).
    RightShift => ">>",

    /// Semicolon (`;`).
    Semicolon => ";",

    /// Slash (`/`).
    Slash => "/",

    /// Slash Equal (`/=`).
    SlashEq => "/=",

    /// Caret (`^`).
    Caret => "^",

    /// Caret Equal (`^=`).
    CaretEq => "^=",

    /// Hash Tag (`#`).
    HashTag => "#"
}

/// Represents token without a specific location in source text.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Display)]
pub enum RawToken {
    /// True boolean literal (`true`).
    #[display(fmt = "`true`")]
    TrueBoolLiteral,
    /// False boolean literal (`false`).
    #[display(fmt = "`false`")]
    FalseBoolLiteral,
    /// Character literal.
    #[display(fmt = "character literal")]
    CharLiteral,
    /// Corresponds to any comment that is not a doc comment.
    #[display(fmt = "comment")]
    Comment,
    /// Module level doc comment.
    #[display(fmt = "global doc comment")]
    GlobalDocComment,
    /// Item doc comment.
    #[display(fmt = "local doc comment")]
    LocalDocComment,
    /// End of file (`\0`).
    #[default]
    #[display(fmt = "end of file")]
    EndOfFile,
    /// Float literal.
    #[display(fmt = "float literal")]
    FloatLiteral,
    /// Identifier.
    #[display(fmt = "identifier")]
    Identifier,
    /// Integer literal.
    #[display(fmt = "integer literal")]
    IntegerLiteral,
    /// Error token.
    #[display(fmt = "{_0}")]
    Error(RawLexError),
    /// Keyword.
    #[display(fmt = "{_0}")]
    Keyword(Keyword),
    /// Punctuator.
    #[display(fmt = "{_0}")]
    Punctuator(Punctuator),
    /// String literal.
    #[display(fmt = "string literal")]
    StringLiteral,
}

impl RawToken {
    #[inline]
    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }
}

impl PartialEq<Punctuator> for RawToken {
    fn eq(&self, other: &Punctuator) -> bool {
        matches!(self, Self::Punctuator(punctuator) if punctuator == other)
    }
}

impl PartialEq<Keyword> for RawToken {
    fn eq(&self, other: &Keyword) -> bool {
        matches!(self, Self::Keyword(keyword) if keyword == other)
    }
}

impl From<RawLexError> for RawToken {
    fn from(value: RawLexError) -> Self {
        Self::Error(value)
    }
}

impl From<Punctuator> for RawToken {
    fn from(value: Punctuator) -> Self {
        Self::Punctuator(value)
    }
}

impl From<Keyword> for RawToken {
    fn from(value: Keyword) -> Self {
        Self::Keyword(value)
    }
}

impl From<RawToken> for String {
    fn from(value: RawToken) -> Self {
        value.to_string()
    }
}

impl RawToken {
    #[inline]
    #[must_use]
    pub const fn eof(&self) -> bool {
        matches!(self, Self::EndOfFile)
    }
}

/// Represents a token with a specified location in source text.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub location: Location,
    pub raw: RawToken,
}

macro_rules! map_precedences {
    { $($($punctuator:ident),* => $precedence:ident,)* } => {
        impl From<Punctuator> for Precedence {
            fn from(value: Punctuator) -> Self {
                match value {
                    $($(| Punctuator::$punctuator)* => Precedence::$precedence,)*
                    _ => Precedence::Lowest
                }
            }
        }
    };
}

map_precedences! {
    DoubleOr => DoubleOr,
    DoubleAmpersand => DoubleAmpersand,
    Or => Or,
    Caret => Xor,
    Eq, PlusEq, MinusEq, AsteriskEq, SlashEq, OrEq, CaretEq, PercentEq => Assign,
    DoubleEq, BangEq, Less, LessEq, Greater, GreaterEq => Comparison,
    LeftShift, RightShift => Shift,
    OpenBracket => GenericArgument,
    Plus, Minus => Sum,
    Asterisk, Slash => Product,
    Percent => Mod,
    OpenParent => Call,
    Dot => Field,
    Tilde, DoublePlus, DoubleMinus, Bang, QuestionMark => Unary,
    OpenBrace => Struct,
}

impl From<RawToken> for Precedence {
    fn from(value: RawToken) -> Self {
        match value {
            RawToken::Punctuator(punctuator) => punctuator.into(),
            RawToken::Keyword(Keyword::As) => Self::As,
            _ => Self::Lowest,
        }
    }
}
