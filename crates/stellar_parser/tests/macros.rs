#[macro_export]
macro_rules! tests_using {
    ($parse_fn:ident, $($name:ident -> $code:expr),*) => {
        #[cfg(test)]
        mod tests {
            use parking_lot::RwLock;
            use stellar_parser::*;
            use stellar_interner::{DUMMY_PATH_ID, IdentifierInterner};
            use stellar_diagnostics::Diagnostics;

            $(
                #[test]
                fn $name() {
                    let identifier_interner = RwLock::new(IdentifierInterner::new());
                    let diagnostics = RwLock::new(Diagnostics::new());

                    let result_ =
                        $parse_fn(DUMMY_PATH_ID, $code, &diagnostics, &identifier_interner);
                    assert!(result_.is_some());
                }
            )*
        }
    };
}
