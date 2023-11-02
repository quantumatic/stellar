//! Provides the [`id_types`] macro.

/// The macro generates types for interned data in database.
///
/// ```ignore
/// id_types! { x, y, z, test }
/// ```
///
/// The macro is used to reduce thousands of lines of boilerplate source code with methods
/// like `Database::add_test()`, `Test::is_valid()`, `Test::get()`, `Test::get_mut()`, and
/// types like `TestId` in `lib.rs`.
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

                impl [<$what:camel Id>] {
                    #[allow(dead_code)]
                    #[doc = "Returns an immutable reference to [`" [<$what:camel Data>] "`] by its ID ([`" [<$what:camel Id>] "`])."]
                    fn get_data(self, db: &Database) -> &[<$what:camel Data>] {
                        &db.package(self.package()).[<$what _>][self.idx() - 1]
                    }

                    #[allow(dead_code)]
                    #[doc = "Returns a mutable reference to [`" [<$what:camel Data>] "`] by its ID ([`" [<$what:camel Id>] "`])."]
                    fn get_data_mut(self, db: &mut Database) -> &mut [<$what:camel Data>] {
                        &mut db.package_mut(self.package()).[<$what _>][self.idx() - 1]
                    }

                    #[doc = "Returns whether a [`" [<$what:camel Data>] "`] with a given ID ([`" [<$what:camel Id>] "`]) is present in the database storage."]
                    #[inline]
                    #[must_use]
                    pub fn is_valid(self, db: &Database) -> bool {
                        if let Some(package) = db.package_or_none(self.package()) {
                            self.idx() - 1 < package.[<$what _>].len()
                        } else {
                            false
                        }
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
                }
            }
        )*
    }
}
