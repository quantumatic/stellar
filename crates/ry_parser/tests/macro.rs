#[macro_export]
macro_rules! test {
    ($name:ident: $source:literal) => {
        #[test]
        #[allow(unused_qualifications)]
        fn $name() {
            let mut diagnostics = vec![];
            let mut interner = ry_interner::Interner::default();
            let state = ry_parser::ParseState::new(&$source, &mut diagnostics, &mut interner);
            let _ = ry_parser::parse_module_using(state);

            assert!(diagnostics.is_empty());
        }
    };
}
