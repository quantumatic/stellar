use std::path::Path;

use ry_workspace::file::SourceFile;

const TEST_SOURCE: &str = "foo\nbar\r\n\nbaz";

#[test]
fn line_starts() {
    let file = SourceFile::new(Path::new("test.ry"), TEST_SOURCE);

    assert_eq!(
        file.line_starts,
        &[
            0,  // "foo\n"
            4,  // "bar\r\n"
            9,  // ""
            10, // "baz"
        ]
    )
}

#[test]
fn line_span_sources() {
    let file = SourceFile::new(Path::new("test.ry"), TEST_SOURCE);

    let line_sources = (0..4)
        .map(|line| {
            let line_range = file.line_range_by_index(line).unwrap();
            &file.source[line_range]
        })
        .collect::<Vec<_>>();

    assert_eq!(line_sources, ["foo\n", "bar\r\n", "\n", "baz"]);
}
