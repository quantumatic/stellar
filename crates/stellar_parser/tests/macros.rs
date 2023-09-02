#[macro_export]
macro_rules! tests_using {
    ($parse_fn:ident, $($name:ident -> $code:expr),*) => {
        #[cfg(test)]
        mod tests {
            use stellar_parser::*;
            use stellar_interner::DUMMY_PATH_ID;
            use stellar_diagnostics::Diagnostics;

            $(
                #[test]
                fn $name() {
                    let mut diagnostics = Diagnostics::new();

                    let result_ =
                        $parse_fn(DUMMY_PATH_ID, $code, &mut diagnostics);
                    assert!(result_.is_some());
                }
            )*
        }
    };
}
