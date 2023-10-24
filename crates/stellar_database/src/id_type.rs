//! Provides the [`id_types`] macro.

/// The macro generates types for interned data in database.
///
/// ```ignore
/// id_types! { x, y, z }
/// ```
///
/// The macro is used to reduce thousands of lines of boilerplate source code with methods
/// like `Database::add_x()`, `Database::contains_x()`, `Database::resolve_x()`, and types like `XId` in `lib.rs`.
macro_rules! id_types {
    {
        $($what:ident),*
    } => {
        $(
            paste! {
                #[doc = "A unique ID that maps to [`" [<$what:camel Data>] "`]."]
                #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
                #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
                pub struct [<$what:camel Id>](
                    // the package that data is associated with
                    PackageId,
                    usize
                );

                pub const [<DUMMY_ $what:upper _ID>]: [<$what:camel Id>] = [<$what:camel Id>](DUMMY_PACKAGE_ID, 0);

                impl [<$what:camel Id>] {
                    /// Constructs a new index type.
                    #[inline]
                    #[must_use]
                    pub fn new(package: PackageId, id: usize) -> Self {
                        Self(package, id)
                    }

                    /// Returns the package ID of the index type.
                    #[inline]
                    #[must_use]
                    pub fn package(&self) -> PackageId {
                        self.0
                    }

                    /// Returns the underlying ID of the index type within the package.
                    #[inline]
                    #[must_use]
                    pub fn idx(&self) -> usize {
                        self.1
                    }
                }

                impl Database {
                    #[doc = "Adds an object of type [`" [<$what:camel Data>] "`] to the database storage and returns its ID ([`" [<$what:camel Id>] "`])."]
                    ///
                    /// # Panics
                    /// Panics if a given package is not present in the database storage.
                    ///
                    /// _This function is automatically generated using a macro!_
                    #[inline]
                    #[must_use]
                    pub fn [<add_ $what>](&mut self, package: PackageId, data: [<$what:camel Data>]) -> [<$what:camel Id>] {
                        self.package_mut(package).[<$what _>].push(data);

                        [<$what:camel Id>](package, self.package(package).[<$what _>].len())
                    }

                    #[doc = "Returns whether a [`" [<$what:camel Data>] "`] with a given ID ([`" [<$what:camel Id>] "`]) is present in the database storage."]
                    ///
                    /// _This function is automatically generated using a macro!_
                    #[inline]
                    #[must_use]
                    pub fn [<contains_ $what>](&self, id: [<$what:camel Id>]) -> bool {
                        if let Some(package) = self.package_or_none(id.package()) {
                            id.idx() - 1 < package.[<$what _>].len()
                        } else {
                            false
                        }
                    }

                    #[doc = "Returns an immutable reference to [`" [<$what:camel Data>] "`] by its ID ([`" [<$what:camel Id>] "`])."]
                    /// # Panics
                    /// Panics if an object with the given ID is not present in the database storage.
                    ///
                    /// _This function is automatically generated using a macro!_
                    #[inline]
                    #[must_use]
                    pub fn [<resolve_ $what >](&self, id: [<$what:camel Id>]) -> &[<$what:camel Data>] {
                        &self.package(id.package()).[<$what _>][id.idx() - 1]
                    }

                    #[doc = "Returns a mutable reference to [`" [<$what:camel Data>] "`] by its ID ([`" [<$what:camel Id>] "`])."]
                    /// # Panics
                    /// Panics if an object with the given ID is not present in the database storage.
                    ///
                    /// _This function is automatically generated using a macro!_
                    #[inline]
                    #[must_use]
                    pub fn [<resolve_ $what _mut>](&mut self, id: [<$what:camel Id>]) -> &mut [<$what:camel Data>] {
                        &mut self.package_mut(id.package()).[<$what _>][id.idx() - 1]
                    }

                    #[doc = "Returns an immutable reference to [`" [<$what:camel Data>] "`] by its ID ([`" [<$what:camel Id>] "`])."]
                    ///
                    /// _This function is automatically generated using a macro!_
                    #[inline]
                    #[must_use]
                    pub fn [<resolve_ $what _or_none>](&self, id: [<$what:camel Id>]) -> Option<&[<$what:camel Data>]> {
                        self.package_or_none(id.package())
                            .and_then(|p| p.[<$what _>].get(id.idx() - 1))
                    }

                    #[doc = "Returns a mutable reference to [`" [<$what:camel Data>] "`] by its ID ([`" [<$what:camel Id>] "`])."]
                    ///
                    /// _This function is automatically generated using a macro!_
                    #[inline]
                    #[must_use]
                    pub fn [<resolve_ $what _mut_or_none>](&mut self, id: [<$what:camel Id>]) -> Option<&mut [<$what:camel Data>]> {
                        self.package_mut_or_none(id.package())
                            .and_then(|p| p.[<$what _>].get_mut(id.idx() - 1))
                    }
                }
            }
        )*
    }
}
