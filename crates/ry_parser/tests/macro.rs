#[macro_export]
macro_rules! test {
    ($name:ident: $source:literal) => {
        #[test]
        #[allow(unused_qualifications)]
        fn $name() {
            let file = ry_span::file::InMemoryFile::new(std::path::Path::new("test.ry"), $source);

            let mut diagnostics = vec![];
            let mut interner = ry_interner::Interner::default();
            let _ = ry_parser::parse_module(0, &file, &mut diagnostics, &mut interner);

            assert!(diagnostics.is_empty());
        }
    };
}
