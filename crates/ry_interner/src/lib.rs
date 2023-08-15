//! The crate implements string internering for Ry programming language
//! compiler. It allows to cache strings and associate them with unique symbols.
//! These allows constant time comparisons and look-ups to underlying interned strings!
//!
//! See the [`Interner`] for more information.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png",
    html_favicon_url = "https://raw.githubusercontent.com/abs0luty/Ry/main/additional/icon/ry.png"
)]
#![warn(missing_docs, clippy::dbg_macro)]
#![warn(
    // rustc lint groups https://doc.rust-lang.org/rustc/lints/groups.html
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
use std::{
    hash::BuildHasherDefault,
    marker::PhantomData,
    path::{Path, PathBuf},
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

extern crate alloc;

use alloc::{string::String, vec::Vec};

use hashbrown::{hash_map::RawEntryMut, HashMap};
use ry_fx_hash::FxHasher;

/// Represents unique symbol corresponding to some interned identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IdentifierID(pub usize);

impl SymbolID for IdentifierID {
    #[inline]
    fn into_storage_index(self) -> usize {
        self.0
    }

    #[inline]
    fn from_storage_index(index: usize) -> Self {
        Self(index)
    }
}

/// A trait, that is implemented for types which represent unique ID-s of
/// interned objects in [`Interner`].
pub trait SymbolID: Copy {
    /// Returns an index of the symbol in the interner memory storage.
    #[must_use]
    fn into_storage_index(self) -> usize;

    /// Returns an interned symbol id from the index in the interner memory storage.
    #[must_use]
    fn from_storage_index(index: usize) -> Self;
}

impl SymbolID for usize {
    #[inline]
    fn into_storage_index(self) -> usize {
        self
    }

    #[inline]
    fn from_storage_index(index: usize) -> Self {
        index
    }
}

impl SymbolID for u64 {
    #[inline]
    fn into_storage_index(self) -> usize {
        usize::try_from(self).unwrap()
    }

    #[inline]
    fn from_storage_index(index: usize) -> Self {
        index as Self
    }
}

impl SymbolID for u32 {
    #[inline]
    fn into_storage_index(self) -> usize {
        usize::try_from(self).unwrap()
    }

    #[inline]
    fn from_storage_index(index: usize) -> Self {
        Self::try_from(index).unwrap()
    }
}

/// # String Interner
///
/// Data structure that allows to resolve/intern strings.
///
/// Interning is a process of storing only a single copy of a particular
/// immutable data value (in this case an identifier), and reusing that copy
/// whenever the same value is encountered again.
///
/// See:
/// - [`Interner::new()`] to create a new empty instance of [`Interner`].
/// - [`Interner::get_or_intern()`] to intern a new string.
/// - [`Interner::resolve()`] to resolve already interned strings.
#[derive(Debug, Clone)]
pub struct Interner<S>
where
    S: SymbolID,
{
    dedup: HashMap<S, (), ()>,
    hasher: BuildHasherDefault<FxHasher>,
    backend: InternerStorage<S>,
}

/// Storage for interned strings.
#[derive(Debug, Clone)]
struct InternerStorage<S>
where
    S: SymbolID,
{
    ends: Vec<usize>,

    /// All interned strings live here.
    storage: String,

    marker: PhantomData<fn() -> S>,
}

impl<S> Default for InternerStorage<S>
where
    S: SymbolID,
{
    fn default() -> Self {
        Self {
            ends: Vec::new(),
            storage: String::new(),
            marker: PhantomData,
        }
    }
}

impl<S> Default for Interner<S>
where
    S: SymbolID,
{
    /// Creates a new empty [`Interner`].
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
fn hash_value<T>(hasher: &impl BuildHasher, value: &T) -> u64
where
    T: ?Sized + Hash,
{
    let state = &mut hasher.build_hasher();
    value.hash(state);
    state.finish()
}

impl<S> InternerStorage<S>
where
    S: SymbolID,
{
    #[must_use]
    #[inline]
    fn with_capacity(capacity: usize) -> Self {
        Self {
            ends: Vec::with_capacity(capacity),
            storage: String::default(),
            marker: PhantomData,
        }
    }

    /// Interns the given string and returns corresponding symbol.
    fn intern(&mut self, string: &str) -> S {
        self.push(string)
    }

    /// Resolves the given symbol to its original string.
    fn resolve(&self, symbol: S) -> Option<&str> {
        self.span_of(symbol).map(|span| self.str_at(span))
    }

    /// Resolves the given symbol to its original string, but without additional checks.
    unsafe fn unchecked_resolve(&self, symbol_id: S) -> &str {
        unsafe { self.str_at(self.unchecked_span_of(symbol_id)) }
    }

    /// Shrink capacity to fit interned symbols exactly.
    fn shrink_to_fit(&mut self) {
        self.ends.shrink_to_fit();
        self.storage.shrink_to_fit();
    }

    /// Returns the index of the next symbol.
    fn next_symbol(&self) -> S {
        S::from_storage_index(self.ends.len())
    }

    /// Returns the span for the given symbol if any.
    fn span_of(&self, symbol_id: S) -> Option<Span> {
        self.ends
            .get(symbol_id.into_storage_index())
            .copied()
            .map(|end| Span {
                start: self
                    .ends
                    .get(symbol_id.into_storage_index().wrapping_sub(1))
                    .copied()
                    .unwrap_or(0),
                end,
            })
    }

    /// Returns the span for the given symbol if any, but without additional checks.
    unsafe fn unchecked_span_of(&self, symbol_id: S) -> Span {
        let end = unsafe { *self.ends.get_unchecked(symbol_id.into_storage_index()) };
        let start = self
            .ends
            .get(symbol_id.into_storage_index().wrapping_sub(1))
            .copied()
            .unwrap_or(0);

        Span { start, end }
    }

    fn str_at(&self, span: Span) -> &str {
        unsafe { from_utf8_unchecked(&self.storage.as_bytes()[span.start..span.end]) }
    }

    /// Pushes the string into the buffer and returns corresponding symbol.
    fn push(&mut self, string: &str) -> S {
        self.storage.push_str(string);

        let end = self.storage.as_bytes().len();
        let symbol = self.next_symbol();

        self.ends.push(end);

        symbol
    }
}

impl<S> Interner<S>
where
    S: SymbolID,
{
    /// Creates a new empty [`Interner`], that only contains builtin symbols.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            dedup: HashMap::default(),
            hasher: BuildHasherDefault::default(),
            backend: InternerStorage::default(),
        }
    }

    /// Creates a new empty `Interner` with the given capacity.
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            dedup: HashMap::with_capacity_and_hasher(capacity, ()),
            hasher: BuildHasherDefault::default(),
            backend: InternerStorage::with_capacity(capacity),
        }
    }

    /// Returns the number of symbols/strings interned by the interner.
    #[inline]
    #[must_use]
    #[allow(clippy::len_without_is_empty)] // interner is never empty
    pub fn len(&self) -> usize {
        self.dedup.len()
    }

    /// Returns the symbol for the given string if it is interned.
    ///
    /// # Example
    /// ```
    /// # use ry_interner::Interner;
    /// let mut interner = Interner::<usize>::default();
    /// let hello_id = interner.get_or_intern("hello");
    /// assert_eq!(Some(hello_id), interner.get("hello"));
    /// ```
    pub fn get(&self, string: impl AsRef<str>) -> Option<S> {
        let string = string.as_ref();
        let hash = hash_value(&self.hasher, string);

        self.dedup
            .raw_entry()
            .from_hash(hash, |symbol| {
                string == unsafe { self.backend.unchecked_resolve(*symbol) }
            })
            .map(|(&symbol, ())| symbol)
    }

    /// Interns the given string and returns a corresponding symbol.
    fn get_or_intern_using<T>(
        &mut self,
        string: T,
        intern_fn: fn(&mut InternerStorage<S>, T) -> S,
    ) -> S
    where
        T: AsRef<str> + Copy + Hash + for<'a> PartialEq<&'a str>,
    {
        let string_ref = string.as_ref();

        let hasher = &self.hasher;
        let hash = hash_value(hasher, string_ref);

        let entry = self.dedup.raw_entry_mut().from_hash(hash, |symbol_id| {
            string_ref == unsafe { self.backend.unchecked_resolve(*symbol_id) }
        });

        let (&mut symbol, &mut ()) = match entry {
            RawEntryMut::Vacant(vacant) => {
                let symbol = intern_fn(&mut self.backend, string);
                vacant.insert_with_hasher(hash, symbol, (), |symbol_id| {
                    hash_value(hasher, unsafe {
                        self.backend.unchecked_resolve(*symbol_id)
                    })
                })
            }
            RawEntryMut::Occupied(occupied) => occupied.into_key_value(),
        };

        symbol
    }

    /// Interns the given string and returns a corresponding symbol.
    #[inline]
    pub fn get_or_intern(&mut self, string: impl AsRef<str>) -> S {
        self.get_or_intern_using(string.as_ref(), InternerStorage::intern)
    }

    /// Shrink backend capacity to fit the interned strings exactly.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.backend.shrink_to_fit();
    }

    /// Returns the string for the given symbol if any.
    ///
    /// # Example
    /// ```
    /// # use ry_interner::Interner;
    /// let mut interner = Interner::<usize>::default();
    /// let hello_id = interner.get_or_intern("hello");
    ///
    /// assert_eq!(interner.get("hello"), Some(hello_id));
    /// assert_eq!(interner.get("!"), None);
    /// ```
    #[inline]
    #[must_use]
    pub fn resolve(&self, symbol: S) -> Option<&str> {
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

/// # Identifier Interner
///
/// Data structure that allows to resolve/intern identifiers. The only
/// difference between identifier interner and [string interner] is that
/// the former contains builtin identifiers (see [`builtin_identifiers`] for more details).
///
/// See:
/// - [`IdentifierInterner::new()`] to create a new empty instance of [`IdentifierInterner`].
/// - [`IdentifierInterner::get_or_intern()`] to intern a new string.
/// - [`IdentifierInterner::resolve()`] to resolve already interned strings.
#[derive(Debug, Clone, Default)]
pub struct IdentifierInterner(Interner<IdentifierID>);

macro_rules! define_builtin_identifiers {
    ($($id_name:ident = $value:tt => $id:tt),+) => {
        /// Defines all builtin identifiers (that are automatically interned by
        /// [`IdentifierInterner`]).
        pub mod builtin_identifiers {
            use crate::IdentifierID;

            $(
                #[doc = concat!("Builtin identifier `", $id, "`.")]
                pub const $id_name: IdentifierID = IdentifierID($value);
            )+
        }

        impl IdentifierInterner {
            /// Creates a new empty [`IdentifierInterner`], that **already contains builtin identifiers**!
            #[must_use]
            pub fn new() -> Self {
                let mut interner = Interner::new();

                $(
                    interner.get_or_intern($id);
                )+

                Self(interner)
            }
        }
    };
}

define_builtin_identifiers! {
    UNDERSCORE = 0 => "_",
    INT8 = 1 => "int8", INT16 = 2 => "int16", INT32 = 3 => "int32", INT64 = 4 => "int64",
    UINT8 = 5 => "uint8", UINT16 = 6 => "uint16", UINT32 = 7 => "uint32", UINT64 = 8 => "uint64",
    FLOAT32 = 9 => "float32", FLOAT64 = 10 => "float64", ISIZE = 11 => "isize",
    USIZE = 12 => "usize", BOOL = 13 => "bool", STRING = 14 => "String", LIST = 15 => "List",
    CHAR = 16 => "char", SMALL_SELF = 17 => "self", BIG_SELF = 18 => "Self",
    SIZE_OF = 19 => "sizeof", STD = 20 => "std"
}

impl IdentifierInterner {
    /// Returns the number of identifiers interned by the interner.
    #[inline]
    #[must_use]
    #[allow(clippy::len_without_is_empty)] // interner is never empty
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns the symbol for the given identifier if it is interned.
    ///
    /// # Example
    /// ```
    /// # use ry_interner::IdentifierInterner;
    /// let mut identifier_interner = IdentifierInterner::new();
    /// let hello_id = identifier_interner.get_or_intern("hello");
    /// assert_eq!(Some(hello_id), identifier_interner.get("hello"));
    /// ```
    #[inline]
    pub fn get(&self, identifier: impl AsRef<str>) -> Option<IdentifierID> {
        self.0.get(identifier)
    }

    /// Interns the given identifier (if it doesn't exist) and returns a corresponding symbol.
    #[inline]
    pub fn get_or_intern(&mut self, identifier: impl AsRef<str>) -> IdentifierID {
        self.0.get_or_intern(identifier)
    }

    /// Shrink backend capacity to fit the interned identifiers exactly.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit();
    }

    /// Returns the string for the given symbol if any.
    ///
    /// # Example
    /// ```
    /// # use ry_interner::{IdentifierInterner, builtin_identifiers::UINT8, IdentifierID};
    /// let mut identifier_interner = IdentifierInterner::new();
    ///
    /// let hello_id = identifier_interner.get_or_intern("hello");
    ///
    /// assert_eq!(identifier_interner.resolve(hello_id), Some("hello"));
    /// assert_eq!(identifier_interner.resolve(UINT8), Some("uint8")); // interned by default
    /// assert_eq!(identifier_interner.resolve(IdentifierID(3123123123)), None);
    /// ```
    #[inline]
    #[must_use]
    pub fn resolve(&self, symbol: IdentifierID) -> Option<&str> {
        self.0.resolve(symbol)
    }
}

/// Storage for file paths (to avoid copying and fast comparing, basically the same
/// movitation as with [`IdentifierInterner`]).
///
/// The ID-s that correspond to file paths have a type of [`PathID`].
#[derive(Debug, Clone)]
pub struct PathInterner(Interner<PathID>);

/// ID of a path in the [`PathInterner`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PathID(pub usize);

impl SymbolID for PathID {
    #[inline]
    fn into_storage_index(self) -> usize {
        self.0
    }

    #[inline]
    fn from_storage_index(index: usize) -> Self {
        Self(index)
    }
}

/// ID of a path, that will never exist in the [`PathInterner`].
pub const DUMMY_PATH_ID: PathID = PathID(0);

impl Default for PathInterner {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl PathInterner {
    /// Creates a new empty file path storage.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(Interner::new())
    }

    /// Adds a path to the interner.
    ///
    /// # Panics
    /// If the path is not a valid UTF-8 string.
    #[inline]
    #[must_use]
    pub fn get_or_intern(&mut self, path: impl AsRef<Path>) -> PathID {
        self.0
            .get_or_intern(path.as_ref().to_str().expect("Invalid UTF-8 path"))
    }

    /// Resolves a path stored in the storage.
    #[inline]
    #[must_use]
    pub fn resolve(&self, id: PathID) -> Option<PathBuf> {
        self.0.resolve(id).map(PathBuf::from)
    }

    /// Resolves a path stored in the storage (same as `resolve_path()`),
    /// but panics if the path is not found.
    #[inline]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn resolve_or_panic(&self, id: PathID) -> PathBuf {
        self.resolve(id)
            .unwrap_or_else(|| panic!("Path with id: {} is not found", id.0))
    }
}
