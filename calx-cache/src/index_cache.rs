use std::marker::PhantomData;
use std::fmt::Debug;
use vec_map::VecMap;

/// A resource cache indexed with an enum type.
///
/// IndexCaches are usually completely pre-filled, but the values must be
/// constructed with some run-time resource, so they can't just be
/// compile-time constants or lazily initialized thread-local variables.
///
/// ```rust
/// #[macro_use] extern crate calx_cache;
///
/// use calx_cache::IndexCache;
///
/// #[derive(Debug, Copy, Clone)]
/// enum CacheItem {
///     ItemOne,
///     ItemTwo
/// }
///
/// cache_key!(CacheItem);
///
/// fn main() {
///     let mut cache: IndexCache<CacheItem, u32> = IndexCache::new();
///     cache.insert(CacheItem::ItemOne, 1);
///     cache.insert(CacheItem::ItemTwo, 2);
///
///     assert_eq!(Some(&1), cache.get(CacheItem::ItemOne));
///     assert_eq!(Some(&2), cache.get(CacheItem::ItemTwo));
/// }
/// ```
pub struct IndexCache<K, V> {
    cache: VecMap<V>,
    // Lock the cache to the key enum type to prevent mismatching keys.
    phantom: PhantomData<K>,
}

impl<K: Debug + Copy + CacheKey, V> IndexCache<K, V> {
    pub fn new() -> IndexCache<K, V> {
        IndexCache {
            cache: VecMap::new(),
            phantom: PhantomData,
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let idx = key.to_usize();
        self.cache.insert(idx, value);
    }

    pub fn get(&self, key: K) -> Option<&V> {
        let idx = key.to_usize();
        self.cache.get(idx)
    }
}

/// The boilerplate trait that must be implemented for IndexCache indexing
/// types (usually enums).
///
/// Use the cache_key! macro.
pub trait CacheKey {
    // XXX: Is there a way to just #derive something for the enum instead so
    // we wouldn't need this and the cache_key! macro?
    fn to_usize(self) -> usize;
}

#[macro_export]
/// Derive CacheKey trait for a given enum to use it with IndexCache.
macro_rules! cache_key {
    ( $name:ident ) => {
        impl $crate::CacheKey for $name { fn to_usize(self) -> usize { self as usize } }
    }
}
