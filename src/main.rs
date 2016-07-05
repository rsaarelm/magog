extern crate euclid;
#[macro_use]
extern crate glium;
extern crate image;
extern crate vitral;

mod backend;

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
pub use euclid::Point2D;
pub use glium::{glutin, DisplayBuild};
pub use backend::Backend;

type Color = [f32; 4];

type ImageRef = usize;

type Splat = Vec<(ImageRef, Point2D<f32>, Color)>;

/// Smart pointer for a static cached resource.
#[derive(Clone)]
struct Resource<T> {
    handle: Rc<T>,
}

impl<T: Sized> AsRef<T> for Resource<T> {
    fn as_ref(&self) -> &T {
        self.handle.as_ref()
    }
}

impl<T: Sized> Deref for Resource<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        self.handle.deref()
    }
}

impl<T: ResourceStore> Resource<T> {
    pub fn new(path: &str) -> Option<Self> {
        if let Some(handle) = ResourceStore::get_resource(path) {
            Some(Resource { handle: handle })
        } else {
            None
        }
    }
}

/// A value that can be aquired given a resource path.
trait Loadable {
    fn load(path: &str) -> Option<Self> where Self: Sized;
}

impl Loadable for image::DynamicImage {
    fn load(path: &str) -> Option<Self> where Self: Sized {
        image::open(path).ok()
    }
}

/// A cache that associates resource values with paths.
///
/// Resources and paths are assumed to be immutable.
struct ResourceCache<T> {
    cache: HashMap<String, Rc<T>>,
}

impl<T: Loadable> ResourceCache<T> {
    pub fn new() -> ResourceCache<T> {
        ResourceCache {
            cache: HashMap::new()
        }
    }

    pub fn get(&mut self, path: &str) -> Option<Rc<T>> {
        if let Some(v) = self.cache.get(path) {
            return Some(v.clone());
        }

        if let Some(v) = T::load(path) {
            let v = Rc::new(v);
            self.cache.insert(path.to_string(), v.clone());
            Some(v)
        } else {
            None
        }
    }

    pub fn insert(&mut self, path: String, value: T) {
        self.cache.insert(path, value);
    }
}

/// A type that implements a singleton resource store.
trait ResourceStore {
    fn get_resource(path: &str) -> Option<Rc<Self>> where Self: Sized;
}


thread_local!(static DYNAMIC_IMAGE: RefCell<ResourceCache<image::DynamicImage>> = RefCell::new(ResourceCache::new()));


impl ResourceStore for image::DynamicImage {
    fn get_resource(path: &str) -> Option<Rc<Self>> {
        DYNAMIC_IMAGE.with(|t| t.borrow_mut().get(path))
    }
}


pub fn main() {
    let display = glutin::WindowBuilder::new()
                      .build_glium()
                      .unwrap();

    let mut backend = Backend::new(&display);

    // Construct Vitral context.
    let mut context: backend::Context;
    let mut builder = vitral::Builder::new();

    let r: Resource<image::DynamicImage> = Resource::new("content/assets/blocks.png").unwrap();
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
