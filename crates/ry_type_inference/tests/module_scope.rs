use ry_interner::Interner;
use ry_type_inference::module_scope::{parse_module_path_using_project_root, Path};

#[test]
pub fn parse_module_path() {
    let mut interner = Interner::default();
    let (a, b, c) = (
        interner.get_or_intern("A"),
        interner.get_or_intern("B"),
        interner.get_or_intern("C"),
    );

    #[cfg(target_os = "windows")]
    assert_eq!(
        parse_module_path_using_project_root("C:\\A\\B\\C.ry", "C:\\A", &mut interner).ok(),
        Some(Path::new(vec![a, b, c])) // A.B.C
    );

    #[cfg(not(target_os = "windows"))]
    assert_eq!(
        parse_module_path_using_project_root("/A/B/C.ry", "/A", &mut interner).ok(),
        Some(Path::new(vec![a, b, c])) // A.B.C
    )
}
