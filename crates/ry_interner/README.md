<p align="center">
    <img width="70%" src="../../additional/icon/banner.png">
</p>

# `ry_interner` crate.

> The crate is a fork of: https://github.com/abs0luty/tiny-interner/.

The crate provides an identifier interner for Ry programming language.

## Why?

The reason why this crate exists is because comparing and copying identifiers takes both **time** and **space**. So instead of storing identifiers as strings, Ry compiler stores them using unique id-s. Here is how it works:

```rs
use ry_interner::Interner;

let mut interner = Interner::default(); // interner is responsible for storing identifiers

let symbol = interner.get_or_intern("foo"); // allocation happens here
let symbol2 = interner.get_or_intern("foo");
let symbol3 = interner.get_or_intern("bar"); // and here

assert_eq!(symbol, symbol2); // comparing identifiers == comparing numbers
assert_ne!(symbol, symbol3);
assert_ne!(symbol2, symbol3);

assert_eq!(symbol + 1, symbol3); // id values are sequential
assert_eq!(symbol3 - 1, symbol2);
```

Here `get_or_intern` function returns value of type `Symbol`:

```rs
...

/// Represents unique symbol corresponding to some interned string.
pub type Symbol = u32;

...
```

Size of `usize` is 4 bytes. But this is ok, because 4 bytes is enough for storing `4294967296` unique identifiers (which is far enough).

Ry also provides builtin symbols that will already be interned when interner will be initialized:

```rs
/// `_` symbol.
pub const UNDERSCORE: Symbol = 0;

/// `int8` symbol.
pub const INT8: Symbol = 1;

/// `int16` symbol.
pub const INT16: Symbol = 2;

...

/// `char` symbol.
pub const CHAR: Symbol = 17;
```

And as long as id values are sequential:

```rs
use ry_interner::Interner;

let mut interner = Interner::default();
let symbol = interner.get_or_intern("foo");

assert_eq!(symbol, CHAR + 1); // symbol == 18 
```

You can also resolve symbols this way:

```rs
assert_eq!(interner.resolve(symbol), Some("foo"));
assert_eq!(interner.resolve(INT8), Some("int8"));
assert_eq!(interner.resolve(symbol + 1), None); // not yet initialized
```
