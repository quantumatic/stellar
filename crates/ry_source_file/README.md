<p align="center">
    <img width="70%" src="../../additional/icon/banner.png">
</p>

# `ry_source_file` crate.

This crate provides utilities for working with Ry source files.

## `Workspace` and `SourceFile`

`Workspace` is a helper struct for working with Ry source files and also provides implementation for `Files` in `codespan_reporting` for proper error reporting. It is important to make sure that you added your source file into the `Workspace`, because it would not report diagnostics properly with ID being out of bonds:

```rs
use std::path::Path;
use ry_source_file::{Workspace, SourceFile};

let workspace = Workspace::new();
let source_file = SourceFile::new(
    Path::new("test.ry"),
    "pub fun main() {}",
);

let file_id = workspace.add_file(source_file);
```

`SourceFile` is a helper struct for working with invidiual Ry source files. Example of how you can use it:

```rs
use std::path::Path;
use ry_source_file::source_file::SourceFile;

let source_file = SourceFile::new(
    Path::new("test.ry"),
    "fun main() {
    println(\"hello, world!\");
}",
);

assert_eq!(source_file.get_line_index_by_byte_index(0), 0);
assert_eq!(source_file.get_line_index_by_byte_index(13), 1);

assert_eq!(source_file.line_range_by_index(0), Some(0..13));
assert_eq!(source_file.line_range_by_index(1), Some(13..43));
assert_eq!(source_file.line_range_by_index(2), Some(43..44));
```

## `Span`

`Span` struct represents a location of some text in the source file. Example of how you can use it:

```rs
use ry_source_file::{span::Span, Workspace};

let workspace = Workspace::new();

let file_id = workspace.add_file(SourceFile::new(Path::new("test.ry"), "pub fun main() {}"));
let fun = workspace.resolve_span(Span::new(4, 7, file_id));
let main = workspace.resolve_span_or_panic(Span::new(8, 12, file_id)); // doesn't return option type

assert_eq!(fun, Some("fun"));
assert_eq!(main, "main");
```
