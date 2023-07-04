use ry_interner::Interner;
use ry_type_inference::module_scope::{parse_module_path, Path};

#[cfg(target_family = "windows")]
#[test]
fn parse_module_path_windows_only() {
    let mut interner = Interner::default();
    let (a, b, c) = (
        interner.get_or_intern("A"),
        interner.get_or_intern("B"),
        interner.get_or_intern("C"),
    );

    assert_eq!(
        parse_module_path("C:\\A\\B\\C.ry", "C:\\A", &mut interner).ok(),
        Some(Path {
            symbols: vec![a, b, c]
        }) // A.B.C
    );
}

#[cfg(target_family = "unix")]
#[test]
fn parse_module_path_on_unix_only() {
    let mut interner = Interner::default();
    let (a, b, c) = (
        interner.get_or_intern("A"),
        interner.get_or_intern("B"),
        interner.get_or_intern("C"),
    );

    assert_eq!(
        parse_module_path("/A/B/C.ry", "/A", &mut interner).ok(),
        Some(Path {
            symbols: vec![a, b, c]
        }) // A.B.C
    )
}
