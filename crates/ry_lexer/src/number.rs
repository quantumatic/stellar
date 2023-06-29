use crate::{is_id_start, Lexer};
use ry_ast::token::{NumberKind, RawLexError, RawToken, Token};
use ry_workspace::span::Span;
use std::char::from_u32;

/// True if `c` is a valid decimal digit.
#[inline]
pub(crate) const fn decimal(c: char) -> bool {
    c.is_ascii_digit()
}

/// True if `c` is a valid hexadecimal digit.
#[inline]
pub(crate) fn hexadecimal(c: char) -> bool {
    c.is_ascii_digit() || ('a'..='f').contains(&c.to_ascii_lowercase())
}

fn invalid_separator(string: &str) -> i32 {
    let mut base = ' ';
    let mut d = '.';
    let mut i = 0;

    let bytes = string.as_bytes();

    if string.len() >= 2 && bytes[0] as char == '0' {
        base = bytes[1] as char;
        if base == 'x' || base == 'o' || base == 'b' {
            d = '0';
            i = 2;
        }
    }

    while i < string.len() {
        let p = d;
        d = bytes[i] as char;
        if d == '_' {
            if p != '0' {
                return TryInto::<i32>::try_into(i).expect("Overflow in Lexer::invalid_separator");
            }
        } else if decimal(d) || base == 'x' && hexadecimal(d) {
            d = '0';
        } else {
            if p == '_' {
                return TryInto::<i32>::try_into(i).expect("Overflow in Lexer::invalid_separator")
                    - 1;
            }

            d = '.';
        }

        i += 1;
    }

    if d == '_' {
        return TryInto::<i32>::try_into(bytes.len())
            .expect("Overflow in Lexer::invalid_separator")
            - 1;
    }

    -1
}

impl Lexer<'_, '_> {
    pub(crate) fn eat_number(&mut self) -> Token {
        let start_location = self.location;

        let mut number_kind = NumberKind::Invalid;

        let mut base: i8 = 10;
        let mut prefix = '0';
        let mut digit_separator = 0;

        let mut invalid_digit_location: Option<usize> = None;

        if self.current != '.' {
            number_kind = NumberKind::Int;

            if self.current == '0' {
                self.advance();

                match self.current.to_ascii_lowercase() {
                    'x' => {
                        self.advance();
                        base = 16;
                        prefix = 'x';
                    }
                    'o' => {
                        self.advance();
                        base = 8;
                        prefix = 'o';
                    }
                    'b' => {
                        self.advance();
                        base = 2;
                        prefix = 'b';
                    }
                    _ => {
                        base = 10;
                        prefix = '0';
                        digit_separator = 1;
                    }
                }
            }

            self.eat_digits(base, &mut invalid_digit_location, &mut digit_separator);
        }

        'processing_float: {
            if self.current == '.' {
                // 1.to_string() is parsed as:
                // Int(1) Punct(Dot) Ident Punct(Lparen) ...
                if is_id_start(self.next) {
                    break 'processing_float;
                }

                number_kind = NumberKind::Float;

                self.advance();

                if prefix == 'o' || prefix == 'b' || prefix == 'x' {
                    return Token {
                        raw: RawToken::Error(RawLexError::InvalidRadixPoint),
                        span: Span::new(start_location, self.location, self.file_id),
                    };
                }

                self.eat_digits(base, &mut invalid_digit_location, &mut digit_separator);
            }

            if digit_separator & 1 == 0 {
                return Token {
                    raw: RawToken::Error(RawLexError::HasNoDigits),
                    span: Span::new(start_location, self.location, self.file_id),
                };
            }
        }

        let l = self.current.to_ascii_lowercase();
        if l == 'e' {
            if prefix != '\0' && prefix != '0' {
                return Token {
                    raw: RawToken::Error(RawLexError::ExponentRequiresDecimalMantissa),
                    span: Span::new(start_location, self.location, self.file_id),
                };
            }

            self.advance();

            number_kind = NumberKind::Float;

            if self.current == '+' || self.current == '-' {
                self.advance();
            }

            let mut ds = 0;
            self.eat_digits(10, &mut None, &mut ds);
            digit_separator |= ds;

            if ds & 1 == 0 {
                return Token {
                    raw: RawToken::Error(RawLexError::ExponentHasNoDigits),
                    span: Span::new(start_location, self.location, self.file_id),
                };
            }
        }

        let string = &self.source[start_location..self.location];

        if let Some(location) = invalid_digit_location {
            if number_kind == NumberKind::Int {
                return Token {
                    raw: RawToken::Error(RawLexError::InvalidDigit),
                    span: Span::new(location, location + 1, self.file_id),
                };
            }
        }

        let s = invalid_separator(string);

        if digit_separator & 2 != 0 && s >= 0 {
            let separator_location =
                TryInto::<usize>::try_into(s).expect("Invalid separator in Lexer::eat_number");
            return Token {
                raw: RawToken::Error(RawLexError::UnderscoreMustSeparateSuccessiveDigits),
                span: Span::new(separator_location, separator_location + 1, self.file_id),
            };
        }

        match number_kind {
            NumberKind::Int => Token {
                raw: RawToken::IntegerLiteral,
                span: Span::new(start_location, self.location, self.file_id),
            },
            NumberKind::Float => Token {
                raw: RawToken::FloatLiteral,
                span: Span::new(start_location, self.location, self.file_id),
            },
            NumberKind::Invalid => {
                unreachable!()
            }
        }
    }

    fn eat_digits(
        &mut self,
        base: i8,
        invalid_digit_location: &mut Option<usize>,
        digit_separator: &mut i32,
    ) {
        if base <= 10 {
            let max = from_u32(
                '0' as u32
                    + TryInto::<u32>::try_into(base).expect("Invalid base in Lexer::eat_digits()"),
            )
            .expect("Invalid max character in Lexer::eat_digits()");

            while decimal(self.current) || self.current == '_' {
                let mut ds = 1;

                if self.current == '_' {
                    ds = 2;
                } else if self.current >= max && invalid_digit_location.is_none() {
                    *invalid_digit_location = Some(self.location);
                }

                *digit_separator |= ds;
                self.advance();
            }
        } else {
            while hexadecimal(self.current) || self.current == '_' {
                let ds = if self.current == '_' { 2 } else { 1 };

                *digit_separator |= ds;
                self.advance();
            }
        }
    }
}
