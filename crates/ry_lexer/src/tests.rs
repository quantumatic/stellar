#[cfg(test)]
mod lexer_tests {
    use crate::Lexer;
    use ry_ast::token::{LexerError, RawToken::*};
    use ry_interner::Interner;

    macro_rules! lexer_test {
        ($name:ident, $contents:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let mut s = Interner::default();
                let mut lexer = Lexer::new($contents.into(), &mut s);
                assert_eq!(lexer.next().unwrap().unwrap(), &$expected);
            }
        };
        ($name:ident, $contents:expr, $expected:expr, $interner:ident) => {
            #[test]
            fn $name() {
                let mut $interner = Interner::default();
                let mut lexer = Lexer::new($contents.into(), &mut $interner);
                assert_eq!(lexer.next().unwrap().unwrap(), &$expected);
            }
        };
    }

    lexer_test!(eof, "", EndOfFile);
    lexer_test!(eof2, " \t\n\r", EndOfFile);

    lexer_test!(
        identifier,
        "test",
        Identifier(interner.get_or_intern("test")),
        interner
    );

    lexer_test!(
        identifier2,
        "тест",
        Identifier(interner.get_or_intern("тест")),
        interner
    );

    lexer_test!(comment, "//test comment", Comment, interner);

    lexer_test!(
        docstring1,
        "///test comment",
        DocstringComment {
            global: false,
            content: "test comment".into()
        },
        interner
    );

    lexer_test!(
        docstring2,
        "//!test comment",
        DocstringComment {
            global: true,
            content: "test comment".into()
        },
        interner
    );

    lexer_test!(
        unexpected_char,
        "#",
        Invalid(LexerError::UnexpectedChar('#'))
    );

    lexer_test!(string, "\"test\"", StringLiteral("test".into()), interner);

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
        Identifier(interner.get_or_intern("test")),
        interner
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
