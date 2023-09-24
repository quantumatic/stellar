macro_rules! __db_first_access_method {
    (
        {
            method_name: $what:ident,
            field_name: $whats:ident,
            entity_id_ty: $id_ty:ty,
            entity_data_ty: $data_ty:ty
        }
    ) => {
            paste! {
                #[doc = "Returns an immutable reference to [`" $data_ty "`] by its ID ([`" $id_ty "`])."]
                #[doc = "# Panics"]
                #[doc = "Panics if an object with the given ID is not present in the database storage."]
                #[doc = ""]
                #[doc = "_This function is automatically generated using a macro!_"]
                #[inline(always)]
                #[must_use]
                pub fn $what(&self, id: $id_ty) -> &$data_ty {
                    &self.packages[id.0.0].$whats[id.1]
                }
            }
    };
    (
        {
            reserved_name,
            method_name: $what:ident,
            field_name: $whats:ident,
            entity_id_ty: $id_ty:ty,
            entity_data_ty: $data_ty:ty
        }
    ) => {
            paste! {
                #[doc = "Returns an immutable reference to [`" $data_ty "`] by its ID ([`" $id_ty "`])."]
                #[doc = "# Panics"]
                #[doc = "Panics if an object with the given ID is not present in the database storage."]
                #[doc = ""]
                #[doc = "_This function is automatically generated using a macro!_"]
                #[inline(always)]
                #[must_use]
                pub fn [<$what _>](&self, id: $id_ty) -> &$data_ty {
                    &self.packages[id.0.0].$whats[id.1]
                }
            }
    };
}

macro_rules! __db_rest_of_access_methods {
    (
        {
            method_name: $what:ident,
            field_name: $whats:ident,
            entity_id_ty: $id_ty:ty,
            entity_data_ty: $data_ty:ty
        }
    ) => {
        paste! {
            #[doc = "Returns a mutable reference to [`" $data_ty "`] by its ID ([`" $id_ty "`])."]
            /// # Panics
            /// Panics if an object with the given ID is not present in the database storage.
            ///
            /// _This function is automatically generated using a macro!_
            #[inline(always)]
            #[must_use]
            pub fn [<$what _mut>](&mut self, id: $id_ty) -> &mut $data_ty {
                &mut self.packages.get_mut(id.0.0).unwrap().$whats[id.1]
            }

            #[doc = "Returns an immutable reference to [`" $data_ty "`] by its ID ([`" $id_ty "`])."]
            ///
            /// _This function is automatically generated using a macro!_
            #[inline(always)]
            #[must_use]
            pub fn [<$what _or_none>](&self, id: $id_ty) -> Option<&$data_ty> {
                self.packages.get(id.0.0).and_then(|p| p.$whats.get(id.1))
            }

            #[doc = "Returns a mutable reference to [`" $data_ty "`] by its ID ([`" $id_ty "`])."]
            ///
            /// _This function is automatically generated using a macro!_
            #[inline(always)]
            #[must_use]
            pub fn [<$what _mut_or_none>](&mut self, id: $id_ty) -> Option<&mut $data_ty> {
                self.packages.get_mut(id.0.0).and_then(|p| p.$whats.get_mut(id.1))
            }

            #[doc = "Returns whether a [`" $data_ty "`] with a given ID ([`" $id_ty "`]) is present in the database storage."]
            ///
            /// _This function is automatically generated using a macro!_
            #[inline(always)]
            #[must_use]
            pub fn [<contains_ $what>](&self, id: $id_ty) -> bool {
                if let Some(package) = self.packages.get(id.0.0) {
                    id.1 < package.$whats.len()
                } else {
                    false
                }
            }

            #[doc = "Adds an object of type [`" $data_ty "`] to the database storage and returns its ID ([`" $id_ty "`])."]
            ///
            /// # Panics
            /// Panics if a given package is not present in the database storage.
            ///
            /// _This function is automatically generated using a macro!_
            #[inline(always)]
            #[must_use]
            pub fn [<add_ $what>](&mut self, package: PackageId, [<$what _>]: $data_ty) -> $id_ty {
                self.packages.get_mut(package.0).unwrap().$whats.push([<$what _>]);

                $id_ty(package, self.packages[package.0].$whats.len() - 1)
            }
        }
    };
    (
        {
            reserved_name,
            method_name: $what:ident,
            field_name: $whats:ident,
            entity_id_ty: $id_ty:ty,
            entity_data_ty: $data_ty:ty
        }
    ) => {
        __db_rest_of_access_methods! {
            {
                method_name: $what,
                field_name: $whats,
                entity_id_ty: $id_ty,
                entity_data_ty: $data_ty
            }
        }
    }
}

macro_rules! define_db_access_methods {
    ($($tt:tt),*) => {
        impl Database {
            $(
                __db_first_access_method! { $tt }
                __db_rest_of_access_methods! { $tt }
            )*
        }
    };
}
