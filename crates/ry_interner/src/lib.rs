//! The crate implements string internering for Ry programming language
//! compiler. It allows to cache strings and associate them with unique symbols.
//! These allows constant time comparisons and look-ups to underlying interned strings!
//!
//! See the [`Interner`] for more information.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png",
    html_favicon_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![warn(missing_docs, clippy::dbg_macro)]
#![deny(
    // rustc lint groups https://doc.rust-lang.org/rustc/lints/groups.html
    warnings,
    future_incompatible,
    let_underscore,
    nonstandard_style,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    unused,
    // rustc allowed-by-default lints https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    non_ascii_idents,
    noop_method_call,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_op_in_unsafe_fn,
    unused_crate_dependencies,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_tuple_struct_fields,
    variant_size_differences,
    // rustdoc lints https://doc.rust-lang.org/rustdoc/lints.html
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_doc_tests,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::bare_urls,
    // clippy categories https://doc.rust-lang.org/clippy/
    clippy::all,
    clippy::correctness,
    clippy::suspicious,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::pedantic,
    clippy::nursery,
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::too_many_lines,
    clippy::option_if_let_else
)]

use core::{
    hash::{BuildHasher, Hash, Hasher},
    str::from_utf8_unchecked,
};

extern crate alloc;

use alloc::{string::String, vec::Vec};

use hashbrown::{
    hash_map::{DefaultHashBuilder, RawEntryMut},
    HashMap,
};

/// Represents unique symbol corresponding to some interned string.
pub type Symbol = usize;

/// Defines all primitive symbols that are interned by default.
pub mod symbols {
    use crate::Symbol;

    /// `_` symbol.
    pub const UNDERSCORE: Symbol = 0;

    /// `int8` symbol.
    pub const INT8: Symbol = 1;

    /// `int16` symbol.
    pub const INT16: Symbol = 2;

    /// `int32` symbol.
    pub const INT32: Symbol = 3;

    /// `int64` symbol.
    pub const INT64: Symbol = 4;

    /// `uint8` symbol.
    pub const UINT8: Symbol = 5;

    /// `uint16` symbol.
    pub const UINT16: Symbol = 6;

    /// `uint32` symbol.
    pub const UINT32: Symbol = 7;

    /// `uint64` symbol.
    pub const UINT64: Symbol = 8;

    /// `float32` symbol.
    pub const FLOAT32: Symbol = 9;

    /// `float64` symbol.
    pub const FLOAT64: Symbol = 10;

    /// `isize` symbol.
    pub const ISIZE: Symbol = 11;

    /// `usize` symbol.
    pub const USIZE: Symbol = 12;

    /// `bool` symbol.
    pub const BOOL: Symbol = 13;

    /// `String` symbol.
    pub const STRING: Symbol = 14;

    /// `List` symbol.
    pub const LIST: Symbol = 15;

    /// `char` symbol.
    pub const CHAR: Symbol = 16;

    /// `self` symbol.
    pub const SMALL_SELF: Symbol = 17;

    /// `Self` symbol.
    pub const BIG_SELF: Symbol = 18;

    /// `sizeof` symbol.
    pub const SIZE_OF: Symbol = 19;

    /// `std` symbol.
    pub const STD: Symbol = 20;
}

/// # Identifier Interner
///
/// Data structure that allows to resolve/intern strings.
///
/// Interning is a process of storing only a single copy of a particular
/// immutable data value (in this case an identifier), and reusing that copy
/// whenever the same value is encountered again.
///
/// See:
/// - [`Interner::default()`] to create a new empty instance of [`Interner`].
/// - [`Interner::get_or_intern()`] to intern a new identifier.
/// - [`Interner::resolve()`] to resolve already interned strings.
#[derive(Debug)]
pub struct Interner<H = DefaultHashBuilder>
where
    H: BuildHasher,
{
    dedup: HashMap<Symbol, (), ()>,
    hasher: H,
    backend: Backend,
}

/// Data structures that organizes interned strings.
#[derive(Debug, Default)]
struct Backend {
    ends: Vec<usize>,
    buffer: String,
}

impl Default for Interner {
    /// Creates a new empty [`Interner`], that only contains builtin symbols.
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

fn hash_value<T>(hasher: &impl BuildHasher, value: &T) -> u64
where
    T: ?Sized + Hash,
{
    let state = &mut hasher.build_hasher();
    value.hash(state);
    state.finish()
}

impl Backend {
    #[must_use]
    #[inline]
    fn with_capacity(capacity: usize) -> Self {
        Self {
            ends: Vec::with_capacity(capacity),
            buffer: String::default(),
        }
    }

    /// Interns the given string and returns corresponding symbol.
    #[inline]
    fn intern(&mut self, string: &str) -> Symbol {
        self.push(string)
    }

    /// Resolves the given symbol to its original string.
    #[inline]
    fn resolve(&self, symbol: Symbol) -> Option<&str> {
        self.span_of(symbol).map(|span| self.str_at(span))
    }

    /// Resolves the given symbol to its original string, but without additional checks.
    #[inline]
    unsafe fn unchecked_resolve(&self, symbol: Symbol) -> &str {
        unsafe { self.str_at(self.unchecked_span_of(symbol)) }
    }

    /// Shrink capacity to fit interned symbols exactly.
    fn shrink_to_fit(&mut self) {
        self.ends.shrink_to_fit();
        self.buffer.shrink_to_fit();
    }

    /// Returns the index of the next symbol.
    fn next_symbol(&self) -> Symbol {
        self.ends.len()
    }

    /// Returns the span for the given symbol if any.
    fn span_of(&self, symbol: Symbol) -> Option<Span> {
        self.ends.get(symbol).copied().map(|end| Span {
            start: self.ends.get(symbol.wrapping_sub(1)).copied().unwrap_or(0),
            end,
        })
    }

    /// Returns the span for the given symbol if any, but without additional checks.
    unsafe fn unchecked_span_of(&self, symbol: Symbol) -> Span {
        let end = unsafe { *self.ends.get_unchecked(symbol) };
        let start = self.ends.get(symbol.wrapping_sub(1)).copied().unwrap_or(0);

        Span { start, end }
    }

    fn str_at(&self, span: Span) -> &str {
        unsafe { from_utf8_unchecked(&self.buffer.as_bytes()[span.start..span.end]) }
    }

    /// Pushes the string into the buffer and returns corresponding symbol.
    fn push(&mut self, string: &str) -> Symbol {
        self.buffer.push_str(string);

        let end = self.buffer.as_bytes().len();
        let symbol = self.next_symbol();

        self.ends.push(end);

        symbol
    }
}

macro_rules! intern_primitive_symbols {
    ($interner:ident, $($name:ident),*) => {
        $(
            $interner.get_or_intern(stringify!($name));
        )*
    }
}

impl<H> Interner<H>
where
    H: BuildHasher + Default,
{
    /// Creates a new empty [`Interner`], that only contains builtin symbols.
    #[must_use]
    #[inline]
    fn new() -> Self {
        let mut interner = Self {
            dedup: HashMap::default(),
            hasher: Default::default(),
            backend: Backend::default(),
        };

        interner.get_or_intern("_");

        intern_primitive_symbols!(
            interner, int8, int16, int32, int64, uint8, uint16, uint32, uint64, float32, float64,
            isize, usize, bool, String, List, char, self, Self, sizeof, STD
        );

        interner
    }

    /// Creates a new empty `Interner` with the given hasher.
    #[inline]
    pub fn with_hasher(hasher: H) -> Self {
        Self {
            dedup: HashMap::default(),
            hasher,
            backend: Backend::default(),
        }
    }

    /// Creates a new empty `Interner` with the given capacity.
    #[must_use]
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            dedup: HashMap::with_capacity_and_hasher(capacity, ()),
            hasher: Default::default(),
            backend: Backend::with_capacity(capacity),
        }
    }

    /// Creates a new empty `Interner` with the given capacity and hasher.
    #[inline]
    pub fn with_capacity_and_hasher(capacity: usize, hasher: H) -> Self {
        Self {
            dedup: HashMap::with_capacity_and_hasher(capacity, ()),
            hasher,
            backend: Backend::with_capacity(capacity),
        }
    }

    /// Returns the number of symbols/strings interned by the interner.
    #[inline]
    #[allow(clippy::len_without_is_empty)] // interner is never empty
    pub fn len(&self) -> usize {
        self.dedup.len()
    }

    /// Returns the symbol for the given string if it is interned.
    ///
    /// # Example
    /// ```
    /// # use ry_interner::Interner;
    /// let mut interner = Interner::default();
    /// let hello_symbol = interner.get_or_intern("hello");
    /// assert_eq!(Some(hello_symbol), interner.get("hello"));
    /// ```
    #[inline]
    pub fn get<T>(&self, string: T) -> Option<Symbol>
    where
        T: AsRef<str>,
    {
        let string_ref = string.as_ref();
        let hasher = &self.hasher;
        let hash = hash_value(hasher, string_ref);

        self.dedup
            .raw_entry()
            .from_hash(hash, |symbol| {
                string_ref == unsafe { self.backend.unchecked_resolve(*symbol) }
            })
            .map(|(&symbol, ())| symbol)
    }

    /// Interns the given string and returns a corresponding symbol.
    #[inline]
    fn get_or_intern_using<T>(
        &mut self,
        string: T,
        intern_fn: fn(&mut Backend, T) -> Symbol,
    ) -> Symbol
    where
        T: AsRef<str> + Copy + Hash + for<'a> PartialEq<&'a str>,
    {
        let string_ref = string.as_ref();

        let hasher = &self.hasher;
        let hash = hash_value(hasher, string_ref);

        let entry = self.dedup.raw_entry_mut().from_hash(hash, |symbol| {
            string_ref == unsafe { self.backend.unchecked_resolve(*symbol) }
        });

        let (&mut symbol, &mut ()) = match entry {
            RawEntryMut::Vacant(vacant) => {
                let symbol = intern_fn(&mut self.backend, string);
                vacant.insert_with_hasher(hash, symbol, (), |symbol| {
                    hash_value(hasher, unsafe { self.backend.unchecked_resolve(*symbol) })
                })
            }
            RawEntryMut::Occupied(occupied) => occupied.into_key_value(),
        };

        symbol
    }

    /// Interns the given string and returns a corresponding symbol.
    #[inline]
    pub fn get_or_intern<T>(&mut self, string: T) -> Symbol
    where
        T: AsRef<str>,
    {
        self.get_or_intern_using(string.as_ref(), Backend::intern)
    }

    /// Shrink backend capacity to fit the interned strings exactly.
    pub fn shrink_to_fit(&mut self) {
        self.backend.shrink_to_fit();
    }

    /// Returns the string for the given symbol if any.
    ///
    /// # Example
    /// ```
    /// # use ry_interner::{Interner, symbols::UINT8};
    /// let mut interner = Interner::default();
    /// let hello_symbol = interner.get_or_intern("hello");
    ///
    /// assert_eq!(interner.get("hello"), Some(hello_symbol));
    /// assert_eq!(interner.get("uint8"), Some(UINT8)); // interned by default
    /// assert_eq!(interner.get("!"), None);
    /// ```
    #[inline]
    pub fn resolve(&self, symbol: Symbol) -> Option<&str> {
        self.backend.resolve(symbol)
    }
}

/// Represents a location of an interned string inside the [`Backend`]'s internal
/// string buffer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Span {
    start: usize,
    end: usize,
}
