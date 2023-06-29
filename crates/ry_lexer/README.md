<p align="center">
    <img width="70%" src="../../additional/icon/banner.png">
</p>

# `ry_lexer` crate.

This crate provides a lexer for Ry programming language.

Lexer is a first stage of compilation, state machine
that converts Ry source text into `Token`s.

Whitespaces are ignored during scanning process.

Lexer is fairly standart. It returns `Token` and then advances its state
on each iteration and stops at eof (always returns `EndOfFile`).

```rs
use ry_lexer::Lexer;
use ry_ast::token::RawToken::EndOfFile;
use ry_interner::Interner;
use ry_workspace::{Span, At};

let mut interner = Interner::default();
let mut lexer = Lexer::new(0, "", &mut interner);
assert_eq!(lexer.next_token(), EndOfFile.at(Span::new(0, 1, 0)));
```

> Note: the Ry lexer makes use of the `ry_interner` crate to perform string interning,
> a process of deduplicating strings, which can be highly beneficial when dealing with
> identifiers.

If error appeared in the process, `Error` token will be returned:

```rs
use ry_lexer::Lexer;
use ry_ast::token::{LexError, RawToken::Error};
use ry_interner::Interner;

let mut interner = Interner::default();
let mut lexer = Lexer::new(0, "١", &mut interner);
assert_eq!(lexer.next_token().unwrap(), &Error(LexError::UnexpectedChar));
```

## Progress:

- [x] Parse identifiers.
- [x] Support for identifier interning.
- [x] Support for escape sequences.
- [x] Parse numbers.
- [x] Parse doc comments and usual comments.
- [x] Support for unicode characters in identifier names ([Unicode Standard Annex #31](https://unicode.org/reports/tr31/)).
- [ ] Write a documentation for everything and refactor the code responsible for parsing numbers.
