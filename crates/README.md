<p align="center"><img width="70%" src="../additional/icon/banner.png" alt="rycon"></p>

An open source programming language for web development with expressive type system and easy-to-learn syntax that makes it easy to build reliable and efficient software.

# Structure of the source code

Ry source code is divided into several crates:

- `ry_ast` - Defines the AST nodes.
- `ry_lexer` - Implements the lexer.
- `ry_parser` - Implements the parser.
- `ry_filesystem` - Implements some utility functions for easier work with the filesystem.
- `ry_interner` - Implements the identifier interner.
- `ry_diagnostics` - Implements the diagnostics.
- `ry_analyze` - Implements the type inference.
