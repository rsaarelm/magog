// !
// Resource caching smart pointer
//

extern crate serde;
extern crate stable_bst;
extern crate compare;

use std::rc::Rc;
use std::ops::Deref;
use stable_bst::{TreeMap, Bound};
use std::fmt;
use std::cmp;
use std::marker::PhantomData;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A type that implements a singleton resource store.
pub trait ResourceStore<K = String> {
    fn get_resource(key: &K) -> Option<Rc<Self>> where Self: Sized;

    fn insert_resource(key: K, resource: Self);

    /// Return next key in the cache in some arbitrary stable iteration order.
    ///
    /// Mostly for internal use.
    fn next_resource_key(prev_key: Option<K>) -> Option<K>;

    /// Return an iterator for all the cached resources.
    fn resource_iter() -> ResourceIter<Self, K>
        where Self: Sized
    {
        ResourceIter {
            next_key: Self::next_resource_key(None),
            phantom: PhantomData,
        }
    }
}

/// Smart pointer for an immutable cached resource.
///
/// The semantics are similar to Rc pointers.
///
/// Resource values will serialize as their key values, so they can be attached to structures that
/// require compact serialization.
#[derive(Clone)]
pub struct Resource<T, K = String> {
    handle: Rc<T>,
    key: K,
}

impl<T: Sized, K> AsRef<T> for Resource<T, K> {
    fn as_ref(&self) -> &T {
        self.handle.as_ref()
    }
}

impl<T: Sized, K> Deref for Resource<T, K> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        self.handle.deref()
    }
}

impl<T> Serialize for Resource<T> {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {
        self.key.serialize(serializer)
    }
}

impl<T: ResourceStore> Deserialize for Resource<T> {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        let key: String = try!(Deserialize::deserialize(deserializer));
        Ok(Self::new(key).unwrap())
    }
}

impl<T, K: cmp::Ord> cmp::Ord for Resource<T, K> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

impl<T, K: cmp::PartialOrd> cmp::PartialOrd for Resource<T, K> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl<T, K: PartialEq> PartialEq for Resource<T, K> {
    fn eq(&self, other: &Self) -> bool {
        self.key.eq(&other.key)
    }
}

impl<T, K: Eq> Eq for Resource<T, K> {}

impl<T, K: fmt::Display> fmt::Display for Resource<T, K> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.key.fmt(formatter)
    }
}

impl<T, K: fmt::Debug> fmt::Debug for Resource<T, K> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.key.fmt(formatter)
    }
}

impl<K, T: ResourceStore<K>> Resource<T, K> {
    pub fn new(key: K) -> Option<Self> {
        if let Some(handle) = ResourceStore::get_resource(&key) {
            Some(Resource {
                handle: handle,
                key: key,
            })
        } else {
            None
        }
    }

    pub fn key<'a>(&'a mut self) -> &'a K {
        &self.key
    }
}


/// A value that can be implicitly constructed given a key.
pub trait Loadable<K = String> {
    fn load(_key: &K) -> Option<Self>
        where Self: Sized
    {
        // Default implementation so that types with no load semantics can be used with
        // ResourceCache so that all inserts must be explicit.
        None
    }
}


/// A cache that associates resource values with paths.
///
/// Resources and paths are assumed to be immutable.
pub struct ResourceCache<T, K: cmp::Ord> {
    cache: TreeMap<K, Rc<T>>,
}

impl<K: Eq + cmp::Ord + Clone, T: Loadable<K>> ResourceCache<T, K> {
    pub fn new() -> ResourceCache<T, K> {
        ResourceCache { cache: TreeMap::new() }
    }

    pub fn get(&mut self, key: &K) -> Option<Rc<T>> {
        if let Some(v) = self.cache.get(key) {
            return Some(v.clone());
        }

        if let Some(v) = T::load(key) {
            let v = Rc::new(v);
            self.cache.insert(key.clone(), v.clone());
            Some(v)
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: K, value: T) {
        self.cache.insert(key, Rc::new(value));
    }

    /// Return next key in the cache in some arbitrary stable iteration order.
    ///
    /// If `prev_key` is `None`, returns first key. Returns `None` if there are no further keys.
    pub fn next_key(&self, prev_key: Option<K>) -> Option<K> {
        if let Some(x) = prev_key {
            self.cache
                .range(Bound::Excluded(&x), Bound::Unbounded)
                .map(|(k, _)| k.clone())
                .next()
        } else {
            self.cache
                .range(Bound::Unbounded, Bound::Unbounded)
                .map(|(k, _)| k.clone())
                .next()
        }
    }
}

/// Implement a thread-local store for a given resource type.
///
/// Name is the identifier for the store, key and value are the types of the resource cache keys
/// and the resources.
///
///     # #[macro_use] extern crate calx_resource;
///     use calx_resource::{Resource, ResourceStore, Loadable};
///
///     // A custom resource type.
///     struct MyAsset { pub text: String }
///
///     // Must be present even if there's no specific implementation.
///     impl Loadable for MyAsset {}
///
///     // Generate a resource store.
///     impl_store!(MY_ASSET_STORE, String, MyAsset);
///
///     fn main() {
///         // Save a resource in the store using a key.
///         MyAsset::insert_resource(
///             "test_resource".to_string(),
///             MyAsset { text: "Hello, world!".to_string() });
///
///         // Fetch a resource handle from the store using our key.
///         let handle: Resource<MyAsset> =
///             Resource::new("test_resource".to_string()).unwrap();
///         assert!(&handle.text == "Hello, world!");
///
///         assert!(MyAsset::resource_iter().next() == Some(handle));
///     }
#[macro_export]
macro_rules! impl_store {
    ($name:ident, $key:ty, $value:ty) => {
    thread_local!(static $name: ::std::cell::RefCell<$crate::ResourceCache<$value, $key>> =
                  ::std::cell::RefCell::new($crate::ResourceCache::new()));

    impl $crate::ResourceStore<$key> for $value {
        fn get_resource(key: &$key) -> Option<::std::rc::Rc<Self>> {
            $name.with(|t| t.borrow_mut().get(key))
        }

        fn insert_resource(key: $key, value: $value) {
            $name.with(|t| t.borrow_mut().insert(key, value));
        }

        fn next_resource_key(prev_key: Option<$key>) -> Option<$key> {
            $name.with(|t| t.borrow_mut().next_key(prev_key))
        }
    }}
}

pub struct ResourceIter<T, K> {
    next_key: Option<K>,
    phantom: PhantomData<T>,
}

impl<T, K> Iterator for ResourceIter<T, K>
    where T: ResourceStore<K>,
          K: Clone
{
    type Item = Resource<T, K>;

    fn next(&mut self) -> Option<Resource<T, K>> {
        let ret;
        if let Some(ref k) = self.next_key {
            ret = Some(Resource::new(k.clone()).unwrap());
        } else {
            return None;
        }
        self.next_key = T::next_resource_key(self.next_key.clone());
        ret
    }
}
