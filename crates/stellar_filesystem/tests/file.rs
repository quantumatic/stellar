use std::path::Path;

use stellar_filesystem::in_memory_file::InMemoryFile;

const TEST_SOURCE: &str = "foo\nbar\r\n\nbaz";

#[test]
fn line_starts() {
    let file = InMemoryFile::new_from_source(Path::new("test.sr"), TEST_SOURCE.to_owned());

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
    let file = InMemoryFile::new_from_source(Path::new("test.sr"), TEST_SOURCE.to_owned());

    let line_sources = (0..4)
        .map(|line| {
            let line_range = file.line_range_by_index(line).unwrap();
            &file.source[line_range]
        })
        .collect::<Vec<_>>();

    assert_eq!(line_sources, ["foo\n", "bar\r\n", "\n", "baz"]);
}
