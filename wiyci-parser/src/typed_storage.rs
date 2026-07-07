// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

macro_rules! typed_storage {
    ($(#[$meta:meta])* $vis:vis $name:ident < $container:ty > { $($variant:ty),+ $(,)? }) => {
        paste::paste! {
            #[allow(unused)]
            $vis trait [<StoredIn $name>]: Sized {
                fn get_storage(storage: &$name) -> &$container<Self>;
                fn get_storage_mut(storage: &mut $name) -> &mut $container<Self>;
            }

            #[derive(Default)]
            $(#[$meta])*
            $vis struct $name {
                $(
                    [<$variant:snake s>]: $container<$variant>,
                )+
            }

            $(
                impl [<StoredIn $name>] for $variant {
                    fn get_storage(storage: &$name) -> &$container<Self> {
                        &storage.[<$variant:snake s>]
                    }

                    fn get_storage_mut(storage: &mut $name) -> &mut $container<Self> {
                        &mut storage.[<$variant:snake s>]
                    }
                }
            )+

            impl $name {
                #[allow(unused)]
                pub fn get<T>(&self) -> &$container<T> where T: [<StoredIn $name>] {
                    T::get_storage(self)
                }

                #[allow(unused)]
                pub fn get_mut<T>(&mut self) -> &mut $container<T> where T: [<StoredIn $name>] {
                    T::get_storage_mut(self)
                }
            }

            impl<T> Extend<T> for $name where T: [<StoredIn $name>], $container<T>: Extend<T> {
                fn extend <I: IntoIterator<Item=T>> (&mut self, iter: I) {
                    T::get_storage_mut(self).extend(iter);
                }
            }

            // TODO: from_inner / into_inner
            // TODO: retain (by kind)
            // TODO: frunk interop
        }
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use std::collections::{BTreeSet, HashSet};

    #[test]
    fn test_get() {
        typed_storage!(MyStorage<Vec>{i32, u32});

        let mut storage = MyStorage::default();
        storage.get_mut().push(1_i32);
        storage.get_mut().push(2_u32);

        assert_eq!(storage.get::<i32>(), &vec![1_i32]);
        assert_eq!(storage.get::<u32>(), &vec![2_u32]);
    }

    #[test]
    fn test_supports_different_collections() {
        {
            typed_storage!(MyStorage<Vec>{u32, i32});
            MyStorage::default().get_mut().push(1_u32);
            MyStorage::default().get_mut().push(2_i32);
        }
        {
            typed_storage!(MyStorage<BTreeSet>{u32, i32});
            MyStorage::default().get_mut().insert(1_u32);
            MyStorage::default().get_mut().insert(2_i32);
        }
        {
            typed_storage!(MyStorage<HashSet>{u32, i32});
            MyStorage::default().get_mut().insert(1_u32);
            MyStorage::default().get_mut().insert(2_i32);
        }
    }

    #[test]
    fn test_extend() {
        {
            typed_storage!(MyStorage<Vec>{u32, i32});
            MyStorage::default().extend(std::iter::once(1_u32));
            MyStorage::default().extend(std::iter::once(2_i32));
        }
        {
            typed_storage!(MyStorage<BTreeSet>{u32, i32});
            MyStorage::default().extend(std::iter::once(1_u32));
            MyStorage::default().extend(std::iter::once(2_i32));
        }
        {
            typed_storage!(MyStorage<HashSet>{u32, i32});
            MyStorage::default().extend(std::iter::once(1_u32));
            MyStorage::default().extend(std::iter::once(2_i32));
        }
    }

    #[test]
    fn test_dump_collection() {
        // collections not supporing advance traits such as Extend stould
        // still be usable, just methods such as extend would be unavailable

        #[derive(Default)]
        struct DumbCollection<T> {
            phantom: std::marker::PhantomData<T>,
        }

        typed_storage!(MyStorage<DumbCollection>{u32, i32});

        MyStorage::default().get_mut::<u32>();
        MyStorage::default().get_mut::<i32>();
    }

    #[test]
    fn test_syntax_visibility() {
        {
            typed_storage!(MyStorage<Vec>{u32, i32});
        }
        {
            typed_storage!(pub MyStorage<Vec>{u32, i32});
        }
    }

    #[test]
    fn test_syntax_meta() {
        typed_storage!(#[derive(Clone)] MyStorage<Vec>{u32, i32});
        let _ = MyStorage::default().clone();
    }

    #[test]
    #[ignore]
    fn test_syntax_complex_types() {
        // not supported due to the way field names are generated
        // may be solved by extending paste (or its successor pastey) crate
        //typed_storage!(#[derive(Clone)] MyStorage<Vec>{Vec<u32>, Vec<i32>});
    }
}
