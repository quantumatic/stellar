#[macro_export]
macro_rules! test {
    ($name:ident: $source:literal) => {
        #[test]
        #[allow(unused_qualifications)]
        fn $name() {
            let source_file = ry_source_file::source_file::SourceFile::new(
                std::path::Path::new("test.ry"),
                $source,
            );

            let mut diagnostics = vec![];
            let mut interner = ry_interner::Interner::default();
            let _ = ry_parser::parse_module(0, &source_file, &mut diagnostics, &mut interner);

            assert!(diagnostics.is_empty());
        }
    };
}
