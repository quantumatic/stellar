#[cfg(test)]
mod lexer_tests {
    use crate::Lexer;
    use ry_ast::token::{LexerError, RawToken::*};
    use string_interner::StringInterner;

    macro_rules! lexer_test {
        ($name:ident, $contents:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let mut s = StringInterner::default();
                let mut lexer = Lexer::new($contents.into(), &mut s);
                assert_eq!(lexer.next().unwrap().value, $expected);
            }
        };
        ($name:ident, $contents:expr, $expected:expr, $string_interner:ident) => {
            #[test]
            fn $name() {
                let mut $string_interner = StringInterner::default();
                let mut lexer = Lexer::new($contents.into(), &mut $string_interner);
                assert_eq!(lexer.next().unwrap().value, $expected);
            }
        };
    }

    lexer_test!(eof, "", EndOfFile);
    lexer_test!(eof2, " \t\n\r", EndOfFile);

    lexer_test!(
        identifier,
        "test",
        Identifier(string_interner.get_or_intern("test")),
        string_interner
    );

    lexer_test!(
        identifier2,
        "тест",
        Identifier(string_interner.get_or_intern("тест")),
        string_interner
    );

    lexer_test!(comment, "//test comment", Comment, string_interner);

    lexer_test!(
        docstring1,
        "///test comment",
        DocstringComment {
            global: false,
            content: "test comment".into()
        },
        string_interner
    );

    lexer_test!(
        docstring2,
        "//!test comment",
        DocstringComment {
            global: true,
            content: "test comment".into()
        },
        string_interner
    );

    lexer_test!(
        unexpected_char,
        "#",
        Invalid(LexerError::UnexpectedChar('#'))
    );

    lexer_test!(
        string,
        "\"test\"",
        String("test".into()),
        string_interner
    );

    lexer_test!(
        string2,
        "\"test",
        Invalid(LexerError::UnterminatedStringLiteral)
    );

    lexer_test!(
        string3,
        "\"test\n",
        Invalid(LexerError::UnterminatedStringLiteral)
    );

    lexer_test!(
        wrapped_id,
        "`test`",
        Identifier(string_interner.get_or_intern("test")),
        string_interner
    );

    lexer_test!(
        wrapped_id2,
        "`test",
        Invalid(LexerError::UnterminatedWrappedIdentifierLiteral)
    );

    lexer_test!(
        wrapped_id3,
        "`test\n",
        Invalid(LexerError::UnterminatedWrappedIdentifierLiteral)
    );
}
