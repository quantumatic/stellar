#[macro_export]
macro_rules! tests_using {
    ($parse_fn:ident, $($name:ident -> $code:expr),*) => {
        #[cfg(test)]
        mod tests {
            use parking_lot::RwLock;
            use stellar_parser::*;
            use stellar_interner::DUMMY_PATH_ID;
            use stellar_diagnostics::Diagnostics;

            $(
                #[test]
                fn $name() {
                    let diagnostics = RwLock::new(Diagnostics::new());

                    let result_ =
                        $parse_fn(DUMMY_PATH_ID, $code, &diagnostics);
                    assert!(result_.is_some());
                }
            )*
        }
    };
}
