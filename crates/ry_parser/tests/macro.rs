#[macro_export]
macro_rules! test {
    ($name:ident: $source:literal) => {
        #[test]
        #[allow(unused_qualifications)]
        fn $name() {
            let mut diagnostics = vec![];
            let mut string_interner = ry_interner::Interner::default();
            let source_file = ry_source_file::source_file::SourceFile::new(
                std::path::Path::new("test.ry"),
                $source,
            );

            let mut cursor =
                ry_parser::Cursor::new(0, &source_file, &mut string_interner, &mut diagnostics);

            cursor.parse();
            assert!(cursor.diagnostics().is_empty());
        }
    };
}
