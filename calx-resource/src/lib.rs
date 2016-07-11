// !
// Resource caching smart pointer
//

extern crate serde;

use std::rc::Rc;
use std::ops::Deref;
use std::hash;
use std::collections::HashMap;
use std::fmt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A type that implements a singleton resource store.
pub trait ResourceStore<K = String> {
    fn get_resource(key: &K) -> Option<Rc<Self>> where Self: Sized;

    fn insert_resource(key: K, resource: Self);
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

impl<T, K: hash::Hash> hash::Hash for Resource<T, K> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
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
}


/// A cache that associates resource values with paths.
///
/// Resources and paths are assumed to be immutable.
pub struct ResourceCache<T, K = String> {
    cache: HashMap<K, Rc<T>>,
}

impl<K: Eq + hash::Hash + Clone, T> ResourceCache<T, K> {
    pub fn new() -> ResourceCache<T, K> {
        ResourceCache { cache: HashMap::new() }
    }

    pub fn get(&mut self, key: &K) -> Option<Rc<T>> {
        self.cache.get(key).map(|x| x.clone())
    }

    pub fn insert(&mut self, key: K, value: T) {
        self.cache.insert(key, Rc::new(value));
    }
}

/// Implement a thread-local store for a given resource type.
///
/// Name is the identifier for the store, key and value are the types of the resource cache keys
/// and the resources.
///
///     # #[macro_use] extern crate calx_resource;
///     use calx_resource::{Resource, ResourceStore};
///
///     // A custom resource type.
///     struct MyResource { pub text: String }
///
///     // Generate a resource store.
///     impl_store!(MY_RESOURCE_STORE, String, MyResource);
///
///     fn main() {
///         // Save a resource in the store using a key.
///         MyResource::insert_resource(
///             "test_resource".to_string(),
///             MyResource { text: "Hello, world!".to_string() });
///
///         // Fetch a resource handle from the store using our key.
///         let handle: Resource<MyResource> =
///             Resource::new("test_resource".to_string()).unwrap();
///         assert!(&handle.text == "Hello, world!");
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
    }}
}
