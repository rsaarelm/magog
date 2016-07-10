extern crate euclid;
#[macro_use]
extern crate glium;
extern crate image;
extern crate vitral;
extern crate serde;
extern crate calx_color;

mod backend;
#[macro_use]
mod resource;

use std::hash;
use image::GenericImage;
use std::collections::HashMap;
use calx_color::color::*;
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

// Brush implements Loadable so we can have a cache for it, but there's no actual implicit load
// method, brushes must be inserted manually in code.
//
// (We *could* make a load method later and have it read configuration files or something that
// specify te brush.)
impl Loadable for Brush {}

impl_store!(BRUSH, String, Brush);

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


struct BrushBuilder<'a, V: 'a> {
    builder: &'a mut vitral::Builder<V>,
    image_file: Option<String>,
    brush: Brush,
    frame_images: HashMap<SubImageSpec, usize>,
}

impl<'a, V: Copy + Eq + 'a> BrushBuilder<'a, V> {
    pub fn new(builder: &'a mut vitral::Builder<V>) -> BrushBuilder<'a, V> {
        BrushBuilder {
            builder: builder,
            image_file: None,
            brush: Vec::new(),
            frame_images: HashMap::new(),
        }
    }

    pub fn file(mut self, name: &str) -> Self {
        self.image_file = Some(name.to_string());
        self
    }

    fn get_frame(&mut self, key: &SubImageSpec) -> usize {
        // XXX: Crashy-crash unwrapping if you feed it bad data.
        if let Some(ret) = self.frame_images.get(key) {
            return *ret;
        }

        let image: Resource<FrameImage, SubImageSpec> = Resource::new(key.clone()).unwrap();
        let ret = self.builder.add_image(&*image);

        self.frame_images.insert(key.clone(), ret);

        ret
    }

    /// Add a new frame with the given bounding rectangle to the current splat.
    pub fn frame(mut self, x: u32, y: u32, w: u32, h: u32) -> Self {
        if self.brush.is_empty() {
            self.brush.push(Vec::new());
        }
        let filename = self.image_file.clone().expect("Image file not set");
        let spec = SubImageSpec::new(filename,
            Rect::new(Point2D::new(x, y), Size2D::new(w, h))).unwrap();

        let image = self.get_frame(&spec);

        let idx = self.brush.len() - 1;
        self.brush[idx].push(Frame { image: image, offset: Point2D::new(0.0, 0.0), color: [1.0, 1.0, 1.0, 1.0] });

        self
    }

    /// Set the color of the last frame.
    pub fn color<C: Into<calx_color::Rgba>>(mut self, c: C) -> Self {
        assert!(!self.brush.is_empty());
        let i = self.brush.len() - 1;
        assert!(!self.brush[i].is_empty());
        let j = self.brush[i].len() - 1;

        let c: calx_color::Rgba = c.into();

        self.brush[i][j].color = [c.r, c.g, c.b, c.a];

        self
    }

    /// Set the offset of the last frame.
    pub fn offset(mut self, x: i32, y: i32) -> Self {
        // Internally offset is floats because everything is, but on the data spec level we're
        // pretty much operating on per-pixel level.

        let i = self.brush.len() - 1;
        assert!(i >= 0);
        let j = self.brush[i].len() - 1;
        assert!(j >= 0);

        self.brush[i][j].offset = Point2D::new(x as f32, y as f32);

        self
    }

    /// Start a new splat in the current brush.
    pub fn splat(mut self) -> Self {
        self.brush.push(Vec::new());
        self
    }

    /// Finish and name the current brush and start a new one.
    pub fn brush(mut self, name: &str) -> Self {
        use std::mem;

        let name = name.to_string();
        // Must be an original name.
        assert!(Brush::get_resource(&name).is_none());

        // Zero cached brush and copy the old value here so we can insert it.
        let mut brush = Vec::new();
        mem::swap(&mut brush, &mut self.brush);
        assert!(!brush.is_empty());

        Brush::insert_resource(name, brush);

        self
    }
}

fn init_brushes<V: Copy + Eq>(builder: &mut vitral::Builder<V>) {
    BrushBuilder::new(builder)
        .file("content/assets/props.png")
        .frame(32, 0, 32, 32).color(RED)
        .brush("cursor")
        ;
}

fn draw_splat(context: &mut backend::Context, offset: Point2D<f32>, splat: &Splat) {
    for frame in splat.iter() {
        context.draw_image(frame.image, offset + frame.offset, frame.color);
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

    init_brushes(&mut builder);

    context = builder.build(|img| backend.make_texture(&display, img));

    let font = context.default_font();

    loop {
        context.begin_frame();

        context.draw_text(font,
                          Point2D::new(4.0, 20.0),
                          [1.0, 1.0, 1.0, 1.0],
                          "Hello, world!");

        let image = Brush::get_resource(&"cursor".to_string()).unwrap()[0][0].image;

        draw_splat(&mut context, Point2D::new(50.0, 50.0), &Brush::get_resource(&"cursor".to_string()).unwrap()[0]);

        if !backend.update(&display, &mut context) {
            return;
        }
    }
}
