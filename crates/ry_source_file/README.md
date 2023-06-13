<p align="center">
    <img width="70%" src="../../additional/icon/banner.png">
</p>

# `ry_source_file` crate.

This crate provides utilities for working with Ry source files.

## `SourceFileManager` and `SourceFile`

`SourceFileManager` is a helper struct for working with Ry source files and also provides implementation for `Files` in `codespan_reporting` for proper error reporting. It is important to make sure that you added your source file into the `SourceFileManager`, because it would not report diagnostics properly with ID being out of bonds:

```rs
use std::path::Path;
use ry_source_file::{SourceFileManager, SourceFile};

let file_manager = SourceFileManager::new();
let source_file = SourceFile::new(
    Path::new("test.ry"),
    "pub fun main() {}",
);

let file_id = file_manager.add_file(source_file);
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
use ry_source_file::{span::Span, SourceFileManager};

let file_manager = SourceFileManager::new();

let file_id = file_manager.add_file(SourceFile::new(Path::new("test.ry"), "pub fun main() {}"));
let fun = file_manager.optionally_resolve_span(Span::new(4, 7, file_id)).unwrap();
let main = file_manager.optionally_resolve_span(Span::new(8, 12, file_id)).unwrap();

assert_eq!(fun, "fun");
assert_eq!(main, "main");
```

If you want to represents an object associated with a particular location:

```rs
//Represents some value that has associated span ([`Span`]) with it.
#[derive(Debug, PartialEq, Clone, Default, Eq, Hash)]
pub struct Spanned<T> {
    //Inner content.
    inner: T,
    //Span.
    span: Span,
}
```

You can use `Spanned` type. For example, where is how token type is defined in `ry_ast`:

```rs
pub type Token = Spanned<RawToken>;
```
