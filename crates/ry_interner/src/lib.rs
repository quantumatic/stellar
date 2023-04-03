#![doc(html_root_url = "https://docs.rs/crate/ry-interner/0.1.0")]
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]
#![warn(unsafe_op_in_unsafe_fn, clippy::redundant_closure_for_method_calls)]

//! 327 lines of Rust code that implement string internering for
//! Ry programming language compiler.
//!
//! The crate caches strings and associates them with unique symbols.
//! These allows constant time comparisons and look-ups to underlying interned strings.
//!
//! ### Examples:
//!
//! Internings:
//! ```
//! use ry_interner::Interner;
//!
//! let mut interner = Interner::default();
//! let symbol0 = interner.get_or_intern("A");
//! let symbol1 = interner.get_or_intern("B");
//! let symbol2 = interner.get_or_intern("C");
//! let symbol3 = interner.get_or_intern("A");
//!
//! assert_ne!(symbol0, symbol1);
//! assert_ne!(symbol0, symbol2);
//! assert_ne!(symbol1, symbol2);
//! assert_eq!(symbol0, symbol3);
//! ```
//!
//! Resolving symbols:
//! ```
//! use ry_interner::Interner;
//!
//! let mut interner = Interner::default();
//! let symbol0 = interner.get_or_intern("A");
//! let symbol1 = interner.get_or_intern("B");
//!
//! assert_eq!(interner.resolve(0), Some("A"));
//! assert_eq!(interner.resolve(1), Some("B"));
//! assert_eq!(interner.resolve(2), None);
//! ```

use core::{
    hash::{BuildHasher, Hash, Hasher},
    marker::PhantomData,
    str::from_utf8_unchecked,
};

extern crate alloc;

use self::alloc::{string::String, vec::Vec};
use hashbrown::{
    hash_map::{DefaultHashBuilder, RawEntryMut},
    HashMap,
};

/// Represents unique symbol corresponding to some interned string.
pub type Symbol = usize;

/// Data structure that allows to resolve/intern strings.
///
/// See:
///  - [`Interner::get_or_intern`] to intern a new string.
///  - [`Interner::resolve`] to resolve already interned strings.
#[derive(Debug)]
pub struct Interner<H = DefaultHashBuilder>
where
    H: BuildHasher,
{
    dedup: HashMap<usize, (), ()>,
    hasher: H,
    backend: Backend,
}

/// Data structures that organizes interned strings.
#[derive(Debug)]
pub struct Backend {
    ends: Vec<usize>,
    buffer: String,
    marker: PhantomData<fn() -> usize>,
}

impl Default for Interner {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Backend {
    #[inline]
    fn default() -> Self {
        Self {
            ends: Vec::default(),
            buffer: String::default(),
            marker: Default::default(),
        }
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
    #[inline]
    fn with_capacity(capacity: usize) -> Self {
        Self {
            ends: Vec::with_capacity(capacity),
            buffer: String::default(),
            marker: Default::default(),
        }
    }

    /// Interns the given string and returns corresponding symbol.
    #[inline]
    fn intern(&mut self, string: &str) -> usize {
        self.push(string)
    }

    /// Interns the given static string and returns corresponding symbol.
    #[inline]
    fn intern_static(&mut self, string: &'static str) -> usize {
        self.intern(string)
    }

    /// Resolves the given symbol to its original string.
    #[inline]
    fn resolve(&self, symbol: usize) -> Option<&str> {
        self.span_of(symbol).map(|span| self.str_at(span))
    }

    /// Resolves the given symbol to its original string, but without additional checks.
    #[inline]
    unsafe fn unchecked_resolve(&self, symbol: usize) -> &str {
        unsafe { self.str_at(self.unchecked_span_of(symbol)) }
    }

    /// Shrink capacity to fit interned symbols exactly.
    fn shrink_to_fit(&mut self) {
        self.ends.shrink_to_fit();
        self.buffer.shrink_to_fit();
    }

    fn next_symbol(&self) -> usize {
        self.ends.len()
    }

    /// Returns the span for the given symbol if any.
    fn span_of(&self, symbol: usize) -> Option<Span> {
        self.ends.get(symbol).copied().map(|end| Span {
            start: self.ends.get(symbol.wrapping_sub(1)).copied().unwrap_or(0),
            end,
        })
    }

    /// Returns the span for the given symbol if any, but without additional checks.
    unsafe fn unchecked_span_of(&self, symbol: usize) -> Span {
        let end = unsafe { *self.ends.get_unchecked(symbol) };
        let start = self.ends.get(symbol.wrapping_sub(1)).copied().unwrap_or(0);

        Span { start, end }
    }

    fn str_at(&self, span: Span) -> &str {
        unsafe {
            from_utf8_unchecked(&self.buffer.as_bytes()[(span.start as usize)..(span.end as usize)])
        }
    }

    /// Pushes the string into the buffer and returns corresponding symbol.
    fn push(&mut self, string: &str) -> usize {
        self.buffer.push_str(string);

        let end = self.buffer.as_bytes().len();
        let symbol = self.next_symbol();

        self.ends.push(end);

        symbol
    }
}

impl<H> Interner<H>
where
    H: BuildHasher + Default,
{
    /// Creates a new empty `Interner`.
    #[inline]
    pub fn new() -> Self {
        Self {
            dedup: HashMap::default(),
            hasher: Default::default(),
            backend: Backend::default(),
        }
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
    pub fn len(&self) -> usize {
        self.dedup.len()
    }

    /// Returns `true` if the string interner has no interned strings/amount of symbols is `0`.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the symbol for the given string if any.
    #[inline]
    pub fn get<T>(&self, string: T) -> Option<usize>
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
        intern_fn: fn(&mut Backend, T) -> usize,
    ) -> usize
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
    pub fn get_or_intern<T>(&mut self, string: T) -> usize
    where
        T: AsRef<str>,
    {
        self.get_or_intern_using(string.as_ref(), Backend::intern)
    }

    /// Interns the given `'static` string and returns a corresponding symbol.
    pub fn get_or_intern_static(&mut self, string: &'static str) -> usize {
        self.get_or_intern_using(string.as_ref(), Backend::intern_static)
    }

    /// Shrink backend capacity to fit the interned strings exactly.
    pub fn shrink_to_fit(&mut self) {
        self.backend.shrink_to_fit();
    }

    /// Returns the string for the given symbol if any.
    #[inline]
    pub fn resolve(&self, symbol: usize) -> Option<&str> {
        self.backend.resolve(symbol)
    }
}

/// Represents a location of an interned string inside the [`Backend::buffer`] buffer.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Span {
    start: usize,
    end: usize,
}
