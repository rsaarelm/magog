extern crate euclid;
#[macro_use]
extern crate glium;
extern crate image;
extern crate vitral;
extern crate serde;

mod backend;

use serde::{Serialize, Serializer, Deserialize, Deserializer};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::hash::Hash;
pub use euclid::Point2D;
pub use glium::{glutin, DisplayBuild};
pub use backend::Backend;

type Color = [f32; 4];

type ImageRef = usize;

type Splat = Vec<(ImageRef, Point2D<f32>, Color)>;

/// Smart pointer for a static cached resource.
#[derive(Clone)]
struct Resource<T, K = String> {
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


impl<K, T: ResourceStore<K>> Resource<T, K> {
    pub fn new(key: K) -> Option<Self> {
        if let Some(handle) = ResourceStore::get_resource(&key) {
            Some(Resource { handle: handle, key: key })
        } else {
            None
        }
    }
}

/// A value that can be aquired given a resource path.
trait Loadable<K = String> {
    fn load(key: &K) -> Option<Self> where Self: Sized {
        // Default implementation so that types with no load semantics can be used with
        // ResourceCache so that all inserts must be explicit.
        None
    }
}

impl Loadable for image::DynamicImage {
    fn load(key: &String) -> Option<Self> where Self: Sized {
        image::open(key).ok()
    }
}

/// A cache that associates resource values with paths.
///
/// Resources and paths are assumed to be immutable.
struct ResourceCache<T, K = String> {
    cache: HashMap<K, Rc<T>>,
}

impl<K: Eq + Hash + Clone, T: Loadable<K>> ResourceCache<T, K> {
    pub fn new() -> ResourceCache<T, K> {
        ResourceCache {
            cache: HashMap::new()
        }
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
}

/// A type that implements a singleton resource store.
trait ResourceStore<K = String> {
    fn get_resource(key: &K) -> Option<Rc<Self>> where Self: Sized;
}


thread_local!(static DYNAMIC_IMAGE: RefCell<ResourceCache<image::DynamicImage>> =
              RefCell::new(ResourceCache::new()));

impl ResourceStore for image::DynamicImage {
    fn get_resource(path: &String) -> Option<Rc<Self>> {
        DYNAMIC_IMAGE.with(|t| t.borrow_mut().get(path))
    }
}


struct SubImageSpec {
    image: Resource<image::DynamicImage>,
    bounds: euclid::Rect<u32>,
}


pub fn main() {
    let display = glutin::WindowBuilder::new()
                      .build_glium()
                      .unwrap();

    let mut backend = Backend::new(&display);

    // Construct Vitral context.
    let mut context: backend::Context;
    let mut builder = vitral::Builder::new();

    let r: Resource<image::DynamicImage> = Resource::new("content/assets/blocks.png".to_string()).unwrap();
    let image = builder.add_image(&*r);

    context = builder.build(|img| backend.make_texture(&display, img));

    let font = context.default_font();

    loop {
        context.begin_frame();

        context.draw_text(font, Point2D::new(4.0, 20.0), [1.0, 1.0, 1.0, 1.0], "Hello, world!");

        context.draw_image(image, Point2D::new(50.0, 50.0), [1.0, 1.0, 1.0, 1.0]);

        if !backend.update(&display, &mut context) {
            return;
        }
    }
}
