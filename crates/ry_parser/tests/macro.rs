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

            let mut state = ry_parser::ParseState::new(0, &source_file);
            let (_, diagnostics, _) = ry_parser::parse_module(&mut state);

            assert!(diagnostics.is_empty());
        }
    };
}
