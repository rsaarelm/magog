extern crate euclid;
#[macro_use]
extern crate glium;
extern crate image;
extern crate vitral;
extern crate serde;

mod backend;
#[macro_use]
mod resource;

use std::hash;
use image::GenericImage;
pub use euclid::{Point2D, Rect, Size2D};
pub use glium::{DisplayBuild, glutin};
pub use backend::Backend;
pub use resource::{Loadable, Resource, ResourceCache, ResourceStore};

type Color = [f32; 4];

type ImageRef = usize;

#[derive(Copy, Clone, Debug)]
struct Frame {
    pub image: ImageRef,
    pub offset: Point2D<f32>,
    pub color: Color,
}

type Splat = Vec<Frame>;

type Brush = Vec<Splat>;

type FrameImage = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;

impl_store!(DYNAMIC_IMAGE, String, image::DynamicImage);

#[derive(Clone, PartialEq, Eq, Debug)]
struct SubImageSpec {
    image: Resource<image::DynamicImage>,
    bounds: euclid::Rect<u32>,
}

impl SubImageSpec {
    pub fn new(path: String, bounds: euclid::Rect<u32>) -> Option<SubImageSpec> {
        if let Some(image) = Resource::new(path) {
            Some(SubImageSpec {
                image: image,
                bounds: bounds,
            })
        } else {
            None
        }
    }
}

impl Loadable<SubImageSpec> for FrameImage {
    fn load(spec: &SubImageSpec) -> Option<Self>
        where Self: Sized
    {
        // XXX: Using sub_image on spec.image would be neater, but can't use it here
        // because current image::SubImage must get a mutable access to the parent image and the
        // resource handle is immutable.
        Some(image::ImageBuffer::from_fn(spec.bounds.size.width, spec.bounds.size.height, |x, y| {
            spec.image.get_pixel(spec.bounds.origin.x + x, spec.bounds.origin.y + y)
        }))
    }
}


impl hash::Hash for SubImageSpec {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.image.hash(state);
        self.bounds.origin.hash(state);
        self.bounds.size.hash(state);
    }
}

impl_store!(FRAME_IMAGE, SubImageSpec, FrameImage);


pub fn main() {
    let display = glutin::WindowBuilder::new()
                      .build_glium()
                      .unwrap();

    let mut backend = Backend::new(&display);

    // Construct Vitral context.
    let mut context: backend::Context;
    let mut builder = vitral::Builder::new();

    let r: Resource<FrameImage, SubImageSpec> =
        Resource::new(SubImageSpec::new("content/assets/props.png".to_string(),
                                        Rect::new(Point2D::new(32, 0), Size2D::new(32, 32)))
                          .unwrap())
            .unwrap();

    let image = builder.add_image(&*r);

    context = builder.build(|img| backend.make_texture(&display, img));

    let font = context.default_font();

    loop {
        context.begin_frame();

        context.draw_text(font,
                          Point2D::new(4.0, 20.0),
                          [1.0, 1.0, 1.0, 1.0],
                          "Hello, world!");

        context.draw_image(image, Point2D::new(50.0, 50.0), [1.0, 1.0, 1.0, 1.0]);

        if !backend.update(&display, &mut context) {
            return;
        }
    }
}
