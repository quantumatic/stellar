#[cfg(test)]
mod tests {
    use stellar_ast::token::{RawLexError, RawToken::*};
    use stellar_interner::{IdentifierInterner, DUMMY_PATH_ID};
    use stellar_lexer::Lexer;

    macro_rules! lexer_test {
        ($name:ident, $source:expr, $expected:pat) => {
            #[test]
            fn $name() {
                use parking_lot::RwLock;

                let identifier_interner = RwLock::new(IdentifierInterner::new());
                let mut lexer = Lexer::new(DUMMY_PATH_ID, $source, &identifier_interner);
                assert!(matches!(lexer.next_token().raw, $expected));
            }
        };
    }

    lexer_test!(identifier, "test", Identifier);
    lexer_test!(identifier2, "тест", Identifier);
    lexer_test!(comment, "//test comment", Comment);
    lexer_test!(global_doc_comment, "///test comment", LocalDocComment);
    lexer_test!(local_doc_comment, "//!test comment", GlobalDocComment);
    lexer_test!(unexpected_char, "١", Error(RawLexError::UnexpectedChar));
    lexer_test!(string, "\"test\"", StringLiteral);
    lexer_test!(
        string2,
        "\"test",
        Error(RawLexError::UnterminatedStringLiteral)
    );
    lexer_test!(
        string3,
        "\"test\n",
        Error(RawLexError::UnterminatedStringLiteral)
    );
    lexer_test!(wrapped_id, "`test`", Identifier);
    lexer_test!(
        wrapped_id2,
        "`test",
        Error(RawLexError::UnterminatedWrappedIdentifier)
    );
    lexer_test!(
        wrapped_id3,
        "`test\n",
        Error(RawLexError::UnterminatedWrappedIdentifier)
    );
    lexer_test!(small_u, "'\\u{1E41}'", CharLiteral);
    lexer_test!(big_u, "\"\\U{0010FFFF}\"", StringLiteral);
}
