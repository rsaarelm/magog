use std::hash;
use std::collections::HashMap;
use std::ops::Deref;
use image::{self, GenericImage};
use euclid::{Point2D, Rect, Size2D};
use vitral;
use calx_resource::{Loadable, Resource, ResourceStore};
use calx_color::Rgba;
use calx_color::color::*;

// XXX: This module is a bit awkward fit to the world crate, which we generally want to keep
// rendering method agnostic. However, world-level types with a visual representation hold on to a
// Brush Resource to represent it, so they need to be exposed to the brush type.

#[derive(Clone)]
struct DynamicImageShim(image::DynamicImage);

impl Loadable for DynamicImageShim {
    fn load(key: &String) -> Option<Self>
        where Self: Sized
    {
        image::open(key).ok().map(|x| DynamicImageShim(x))
    }
}

impl_store!(DYNAMIC_IMAGE, String, DynamicImageShim);

pub type Color = [f32; 4];

pub type ImageRef = usize;

#[derive(Copy, Clone, Debug)]
pub struct Splat {
    pub image: ImageRef,
    pub offset: Point2D<f32>,
    pub color: Color,
}

pub type Frame = Vec<Splat>;

#[derive(Clone)]
pub struct Brush(pub Vec<Frame>);

impl Deref for Brush {
    type Target = Vec<Frame>;

    #[inline(always)]
    fn deref(&self) -> &Vec<Frame> {
        &self.0
    }
}

// Brush implements Loadable so we can have a cache for it, but there's no actual implicit load
// method, brushes must be inserted manually in code.
//
// (We *could* make a load method later and have it read configuration files or something that
// specify te brush.)
impl Loadable for Brush {}

impl_store!(BRUSH, String, Brush);

type SplatImage = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;


#[derive(Clone, PartialEq, Eq, Debug)]
struct SubImageSpec {
    image: Resource<DynamicImageShim>,
    bounds: Rect<u32>,
}

impl SubImageSpec {
    pub fn new(path: String, bounds: Rect<u32>) -> Option<SubImageSpec> {
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

impl Loadable<SubImageSpec> for SplatImage {
    fn load(spec: &SubImageSpec) -> Option<Self>
        where Self: Sized
    {
        // XXX: Using sub_image on spec.image would be neater, but can't use it here
        // because current image::SubImage must get a mutable access to the parent image and the
        // resource handle is immutable.
        Some(image::ImageBuffer::from_fn(spec.bounds.size.width, spec.bounds.size.height, |x, y| {
            spec.image.0.get_pixel(spec.bounds.origin.x + x, spec.bounds.origin.y + y)
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

impl_store!(SPLAT_IMAGE, SubImageSpec, SplatImage);


pub struct BrushBuilder<'a, V: 'a> {
    builder: &'a mut vitral::Builder<V>,
    image_file: Option<String>,
    brush: Vec<Frame>,
    splat_images: HashMap<SubImageSpec, usize>,
    color: Rgba,
}

impl<'a, V: Copy + Eq + 'a> BrushBuilder<'a, V> {
    pub fn new(builder: &'a mut vitral::Builder<V>) -> BrushBuilder<'a, V> {
        BrushBuilder {
            builder: builder,
            image_file: None,
            brush: Vec::new(),
            splat_images: HashMap::new(),
            color: WHITE,
        }
    }

    pub fn file(mut self, name: &str) -> Self {
        self.image_file = Some(name.to_string());
        self
    }

    fn get_splat(&mut self, key: &SubImageSpec) -> usize {
        // XXX: Crashy-crash unwrapping if you feed it bad data.
        if let Some(ret) = self.splat_images.get(key) {
            return *ret;
        }

        let image: Resource<SplatImage, SubImageSpec> = Resource::new(key.clone()).unwrap();
        let ret = self.builder.add_image(&*image);

        self.splat_images.insert(key.clone(), ret);

        ret
    }

    /// Add a new splat with the given bounding rectangle to the current frame.
    pub fn splat(mut self, x: u32, y: u32, w: u32, h: u32) -> Self {
        if self.brush.is_empty() {
            self.brush.push(Vec::new());
        }
        let filename = self.image_file.clone().expect("Image file not set");
        let spec = SubImageSpec::new(filename, Rect::new(Point2D::new(x, y), Size2D::new(w, h)))
                       .unwrap();

        let image = self.get_splat(&spec);

        let idx = self.brush.len() - 1;
        self.brush[idx].push(Splat {
            image: image,
            offset: Point2D::new(0.0, 0.0),
            color: [self.color.r, self.color.g, self.color.b, self.color.a],
        });

        self
    }

    /// Set the color for the next splats.
    pub fn color<C: Into<Rgba>>(mut self, c: C) -> Self {
        self.color = c.into();
        self
    }

    /// Set the offset of the last splat.
    pub fn offset(mut self, x: i32, y: i32) -> Self {
        assert!(!self.brush.is_empty());
        let i = self.brush.len() - 1;
        assert!(!self.brush[i].is_empty());
        let j = self.brush[i].len() - 1;

        // Internally offset is floats because all screen geometry stuff is, but on the data spec
        // level we're pretty much operating on per-pixel level.
        self.brush[i][j].offset = Point2D::new(x as f32, y as f32);

        self
    }

    /// Helper for regular tiles.
    pub fn tile(self, x: u32, y: u32) -> Self {
        self.splat(x, y, 32, 32).offset(16, 16)
    }

    /// Helper for block chunks.
    ///
    /// Blocks are built from three 96x32 strips. First one contains the vertical edges, the second
    /// contains the rear block and the third contains the block front. The vertical and rear
    /// frames are nondescript and will probably be reused extensively.
    ///
    /// Block shaping is somewhat complicated and requires a large number of frames.
    pub fn block(self, vert_x: u32, vert_y: u32, rear_x: u32, rear_y: u32, x: u32, y: u32) -> Self {
        self.splat(vert_x, vert_y, 16, 32).offset(16, 16)               // 0: Top left
            .frame().splat(vert_x + 16, vert_y, 16, 32).offset(0, 16)   // 1: Top right
            .frame().splat(vert_x + 32, vert_y, 16, 32).offset(16, 16)  // 2: Middle left
            .frame().splat(vert_x + 48, vert_y, 16, 32).offset(0, 16)   // 3: Middle right
            .frame().splat(vert_x + 64, vert_y, 16, 32).offset(16, 16)  // 4: Bottom left
            .frame().splat(vert_x + 80, vert_y, 16, 32).offset(0, 16)   // 5: Bottom right

            .frame().splat(rear_x, rear_y, 10, 32).offset(16, 16)       // 6: Left half

            .frame().splat(rear_x + 10, rear_y, 6, 32).offset(6, 16)    // 7: Front
            .frame().splat(rear_x + 16, rear_y, 6, 32).offset(0, 16)    // 8

            .frame().splat(rear_x + 22, rear_y, 10, 32).offset(-6, 16)  // 9: Right half

            .frame().splat(rear_x + 32, rear_y, 10, 32).offset(16, 16)  // 10: Y-axis slope
            .frame().splat(rear_x + 42, rear_y, 6, 32).offset(6, 16)    // 11
            .frame().splat(rear_x + 48, rear_y, 6, 32).offset(0, 16)    // 12
            .frame().splat(rear_x + 54, rear_y, 10, 32).offset(-6, 16)  // 13

            .frame().splat(rear_x + 64, rear_y, 10, 32).offset(16, 16)  // 14: X-axis slope
            .frame().splat(rear_x + 74, rear_y, 6, 32).offset(6, 16)    // 15
            .frame().splat(rear_x + 80, rear_y, 6, 32).offset(0, 16)    // 16
            .frame().splat(rear_x + 86, rear_y, 10, 32).offset(-6, 16)  // 17

            .frame().splat(x, y, 10, 32).offset(16, 16)                 // 18 Left half

            .frame().splat(x + 10, y, 6, 32).offset(6, 16)              // 19: Front
            .frame().splat(x + 16, y, 6, 32).offset(0, 16)              // 20

            .frame().splat(x + 22, y, 10, 32).offset(-6, 16)            // 21: Right half

            .frame().splat(x + 32, y, 10, 32).offset(16, 16)            // 22: Y-axis slope
            .frame().splat(x + 42, y, 6, 32).offset(6, 16)              // 23
            .frame().splat(x + 48, y, 6, 32).offset(0, 16)              // 24
            .frame().splat(x + 54, y, 10, 32).offset(-6, 16)            // 25

            .frame().splat(x + 64, y, 10, 32).offset(16, 16)            // 26: X-axis slope
            .frame().splat(x + 74, y, 6, 32).offset(6, 16)              // 27
            .frame().splat(x + 80, y, 6, 32).offset(0, 16)              // 28
            .frame().splat(x + 86, y, 10, 32).offset(-6, 16)            // 29
    }

    /// Helper for wall tiles
    ///
    /// Wall tiles are chopped up from two 32x32 images. One contains the center pillar wallform
    /// and the other contains the two long sides wallform.
    pub fn wall(self, center_x: u32, center_y: u32, sides_x: u32, sides_y: u32) -> Self {
        self.splat(center_x, center_y, 16, 32).offset(16, 16)
            .frame().splat(center_x + 16, center_y, 16, 32).offset(0, 16)
            .frame().splat(sides_x, sides_y, 16, 32).offset(16, 16)
            .frame().splat(sides_x + 16, sides_y, 16, 32).offset(0, 16)
    }

    /// Start a new frame in the current brush.
    pub fn frame(mut self) -> Self {
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

        Brush::insert_resource(name, Brush(brush));

        // Reset color
        self.color = WHITE;

        self
    }
}
