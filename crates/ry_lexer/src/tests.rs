#[cfg(test)]
mod tests {
    use ry_ast::token::{LexError, RawToken::*};

    macro_rules! lexer_test {
        ($name:ident, $contents:expr, $expected:pat) => {
            #[test]
            fn $name() {
                let mut interner = ry_interner::Interner::default();
                let mut lexer = crate::Lexer::new($contents.into(), &mut interner);
                assert!(matches!(lexer.next().unwrap().unwrap(), &$expected));
            }
        };
    }

    lexer_test!(identifier, "test", Identifier(..));
    lexer_test!(identifier2, "тест", Identifier(..));
    lexer_test!(comment, "//test comment", Comment);
    lexer_test!(global_doc_comment, "///test comment", LocalDocComment);
    lexer_test!(local_doc_comment, "//!test comment", GlobalDocComment);
    lexer_test!(unexpected_char, "#", Error(LexError::UnexpectedChar));
    lexer_test!(string, "\"test\"", StringLiteral);
    lexer_test!(
        string2,
        "\"test",
        Error(LexError::UnterminatedStringLiteral)
    );
    lexer_test!(
        string3,
        "\"test\n",
        Error(LexError::UnterminatedStringLiteral)
    );
    lexer_test!(wrapped_id, "`test`", Identifier(..));
    lexer_test!(
        wrapped_id2,
        "`test",
        Error(LexError::UnterminatedWrappedIdentifier)
    );
    lexer_test!(
        wrapped_id3,
        "`test\n",
        Error(LexError::UnterminatedWrappedIdentifier)
    );
}
