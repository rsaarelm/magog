extern crate euclid;
#[macro_use]
extern crate glium;
extern crate image;
extern crate vitral;
extern crate serde;

mod backend;
mod resource;

use std::rc::Rc;
use std::cell::RefCell;
pub use euclid::Point2D;
pub use glium::{glutin, DisplayBuild};
pub use backend::Backend;
pub use resource::{Resource, ResourceStore, ResourceCache};

type Color = [f32; 4];

type ImageRef = usize;

type Splat = Vec<(ImageRef, Point2D<f32>, Color)>;


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
