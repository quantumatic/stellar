#[cfg(test)]
mod lexer_tests {
    use crate::Lexer;
    use ry_ast::token::*;

    macro_rules! def_lex {
        ($l: ident, $contents: expr) => {
            let content = $contents.to_owned();
            let mut $l = Lexer::new(&content);
        };
    }

    #[test]
    fn eof_test() {
        def_lex!(l, "");
        assert_eq!(l.next().unwrap().value, RawToken::EndOfFile);
    }

    #[test]
    fn eof2_test() {
        def_lex!(l, " \t\n\r");
        assert_eq!(l.next().unwrap().value, RawToken::EndOfFile);
    }

    #[test]
    fn identifier_test() {
        def_lex!(l, "test");
        assert_eq!(
            l.next().unwrap().value,
            RawToken::Identifier("test".to_owned())
        );
    }

    #[test]
    fn identifier2_test() {
        def_lex!(l, "привет");
        assert_eq!(
            l.next().unwrap().value,
            RawToken::Identifier("привет".to_owned())
        );
    }

    #[test]
    fn comment_test() {
        def_lex!(l, "//test comment");
        assert_eq!(
            l.next().unwrap().value,
            RawToken::Comment("test comment".to_owned())
        );
    }

    #[test]
    fn unexpected_char_test() {
        def_lex!(l, "#");
        assert_eq!(
            l.next().unwrap().value,
            RawToken::Invalid(LexerError::UnexpectedChar('#'))
        );
    }

    #[test]
    fn string_test() {
        def_lex!(l, "\"test\"");
        assert_eq!(l.next().unwrap().value, RawToken::String("test".to_owned()));
    }

    #[test]
    fn string2_test() {
        def_lex!(l, "\"test");
        assert_eq!(
            l.next().unwrap().value,
            RawToken::Invalid(LexerError::UnterminatedStringLiteral)
        );
    }

    #[test]
    fn string3_test() {
        def_lex!(l, "\"test\n");
        assert_eq!(
            l.next().unwrap().value,
            RawToken::Invalid(LexerError::UnterminatedStringLiteral)
        );
    }

    #[test]
    fn wrapped_id_test() {
        def_lex!(l, "`test`");
        assert_eq!(
            l.next().unwrap().value,
            RawToken::Identifier("test".to_owned())
        );
    }

    #[test]
    fn wrapped_id2_test() {
        def_lex!(l, "`test");
        assert_eq!(
            l.next().unwrap().value,
            RawToken::Invalid(LexerError::UnterminatedWrappedIdentifierLiteral)
        );
    }

    #[test]
    fn wrapped_id3_test() {
        def_lex!(l, "`test\n");
        assert_eq!(
            l.next().unwrap().value,
            RawToken::Invalid(LexerError::UnterminatedWrappedIdentifierLiteral)
        );
    }

    #[test]
    fn number_test() {
        def_lex!(l, "12345");
        assert_eq!(l.next().unwrap().value, RawToken::Int(12345));
    }

    #[test]
    fn number2_test() {
        def_lex!(l, "12345.12345");
        assert_eq!(l.next().unwrap().value, RawToken::Float(12345.12345));
    }

    #[test]
    fn number3_test() {
        def_lex!(l, "12345.");
        assert_eq!(l.next().unwrap().value, RawToken::Float(12345.));
    }

    #[test]
    fn number4_test() {
        def_lex!(l, "1e3");
        assert_eq!(l.next().unwrap().value, RawToken::Float(1e3));
    }

    #[test]
    fn number5_test() {
        def_lex!(l, "0b");
        assert_eq!(
            l.next().unwrap().value,
            RawToken::Invalid(LexerError::HasNoDigits)
        );
    }

    #[test]
    fn number6_test() {
        def_lex!(l, "12.3e");
        assert_eq!(
            l.next().unwrap().value,
            RawToken::Invalid(LexerError::ExponentHasNoDigits)
        );
    }

    #[test]
    fn number7_test() {
        def_lex!(l, "0x0.");
        assert_eq!(
            l.next().unwrap().value,
            RawToken::Invalid(LexerError::InvalidRadixPoint)
        );
    }

    #[test]
    fn number8_test() {
        def_lex!(l, "2.7_e0");
        assert_eq!(
            l.next().unwrap().value,
            RawToken::Invalid(LexerError::UnderscoreMustSeperateSuccessiveDigits)
        );
    }

    #[test]
    fn number9_test() {
        def_lex!(l, "0b__0");
        assert_eq!(
            l.next().unwrap().value,
            RawToken::Invalid(LexerError::UnderscoreMustSeperateSuccessiveDigits)
        );
    }

    #[test]
    fn number10_test() {
        def_lex!(l, "0o60___0");
        assert_eq!(
            l.next().unwrap().value,
            RawToken::Invalid(LexerError::UnderscoreMustSeperateSuccessiveDigits)
        );
    }

    #[test]
    fn number11_test() {
        def_lex!(l, "10e+12_i");
        assert_eq!(
            l.next().unwrap().value,
            RawToken::Invalid(LexerError::UnderscoreMustSeperateSuccessiveDigits)
        );
    }

    #[test]
    fn number12_test() {
        def_lex!(l, "0._1");
        assert_eq!(
            l.next().unwrap().value,
            RawToken::Invalid(LexerError::UnderscoreMustSeperateSuccessiveDigits)
        );
    }

    #[test]
    fn number13_test() {
        def_lex!(l, "0b1101");
        assert_eq!(l.next().unwrap().value, RawToken::Int(13));
    }

    #[test]
    fn number14_test() {
        def_lex!(l, "0x9b");
        assert_eq!(l.next().unwrap().value, RawToken::Int(155));
    }

    #[test]
    fn op_test() {
        def_lex!(l, "+");
        assert_eq!(l.next().unwrap().value, RawToken::Plus);
    }

    #[test]
    fn op2_test() {
        def_lex!(l, "++");
        assert_eq!(l.next().unwrap().value, RawToken::PlusPlus);
        assert_eq!(l.next().unwrap().value, RawToken::EndOfFile);
    }
}
