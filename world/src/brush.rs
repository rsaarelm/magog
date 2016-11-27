use std::hash;
use std::collections::HashMap;
use image::{self, GenericImage};
use euclid::{Point2D, Rect, Size2D};
use vitral::{ImageBuffer, ImageData};
use vitral_atlas;
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

/// Brush element types parametrized on image type.
pub mod generic {
    use std::ops::Deref;
    use euclid::Point2D;
    use Color;

    #[derive(Clone)]
    pub struct Splat<T: Clone> {
        pub image: T,
        pub offset: Point2D<f32>,
        pub color: Color,
    }

    pub type Frame<T> = Vec<Splat<T>>;

    #[derive(Clone)]
    pub struct Brush<T: Clone>(pub Vec<Frame<T>>);

    impl<T: Clone> Brush<T> {
        pub fn new(data: Vec<Frame<T>>) -> Brush<T> { Brush(data) }

        /// Convert the image type using the given function.
        pub fn map<F, U>(self, f: F) -> Brush<U>
            where F: Fn(T) -> U,
                  U: Clone
        {
            Brush(self.0
                      .into_iter()
                      .map(|a| {
                          a.into_iter()
                           .map(|b| {
                               Splat {
                                   image: f(b.image),
                                   offset: b.offset,
                                   color: b.color,
                               }
                           })
                           .collect()
                      })
                      .collect())
        }
    }

    impl<T: Clone> Deref for Brush<T> {
        type Target = Vec<Frame<T>>;

        #[inline(always)]
        fn deref(&self) -> &Vec<Frame<T>> { &self.0 }
    }
}

// XXX: Hardcoded assumption here that the Vitral backend uses `usize` as the texture handle type.

pub type Splat = generic::Splat<ImageData<usize>>;

pub type Frame = generic::Frame<ImageData<usize>>;

pub type Brush = generic::Brush<ImageData<usize>>;


// Brush implements Loadable so we can have a cache for it, but there's no actual implicit load
// method, brushes must be inserted manually in code.
//
// (We *could* make a load method later and have it read configuration files or something that
// specify te brush.)
impl<T: Clone> Loadable for generic::Brush<T> {}

impl_store!(BRUSH, String, Brush);


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

impl Loadable<SubImageSpec> for ImageBuffer {
    fn load(spec: &SubImageSpec) -> Option<Self>
        where Self: Sized
    {
        let build_fn = |x, y| {
            // TODO: Figure out a non-unsafe API for converting image Rgba to vitral::ImageBuffer
            // u32.
            unsafe {
                use std::mem::transmute;
                transmute::<image::Rgba<u8>, u32>(spec.image.0.get_pixel(spec.bounds.origin.x + x,
                                                                         spec.bounds.origin.y + y))
            }
        };

        Some(ImageBuffer::from_fn(spec.bounds.size.width, spec.bounds.size.height, build_fn))
    }
}


impl hash::Hash for SubImageSpec {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.image.hash(state);
        self.bounds.origin.hash(state);
        self.bounds.size.hash(state);
    }
}

impl_store!(SPLAT_IMAGE, SubImageSpec, ImageBuffer);


pub struct BrushBuilder {
    image_file: Option<String>,
    current_brush: Vec<generic::Frame<usize>>,
    splat_images: HashMap<SubImageSpec, usize>,
    color: Rgba,
    brushes: HashMap<String, generic::Brush<usize>>,
}

impl BrushBuilder {
    pub fn new() -> BrushBuilder {
        BrushBuilder {
            image_file: None,
            current_brush: Vec::new(),
            splat_images: HashMap::new(),
            color: WHITE,
            brushes: HashMap::new(),
        }
    }

    pub fn file(mut self, name: &str) -> Self {
        self.image_file = Some(name.to_string());
        self
    }

    fn get_splat(&mut self, key: &SubImageSpec) -> usize {
        // Are we reusing an existing image? Then just return the index.
        if let Some(ret) = self.splat_images.get(key) {
            return *ret;
        }

        // A new image spec was encountered. This needs to be a new item in the atlas, so add it to
        // the end of the queue.
        let ret = self.splat_images.len();
        self.splat_images.insert(key.clone(), ret);
        ret
    }

    /// Add a new splat with the given bounding rectangle to the current frame.
    pub fn splat(mut self, x: u32, y: u32, w: u32, h: u32) -> Self {
        if self.current_brush.is_empty() {
            self.current_brush.push(Vec::new());
        }
        let filename = self.image_file.clone().expect("Image file not set");
        let spec = SubImageSpec::new(filename, Rect::new(Point2D::new(x, y), Size2D::new(w, h)))
                       .expect(&format!("Failed to load image {:?}", &self.image_file));

        let image = self.get_splat(&spec);

        let idx = self.current_brush.len() - 1;
        self.current_brush[idx].push(generic::Splat {
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
        assert!(!self.current_brush.is_empty());
        let i = self.current_brush.len() - 1;
        assert!(!self.current_brush[i].is_empty());
        let j = self.current_brush[i].len() - 1;

        // Internally offset is floats because all screen geometry stuff is, but on the data spec
        // level we're pretty much operating on per-pixel level.
        self.current_brush[i][j].offset = Point2D::new(x as f32, y as f32);

        self
    }

    /// Add bobbing idle animation using the last frame.
    pub fn bob(mut self) -> Self {
        assert!(!self.current_brush.is_empty());
        let mut next_frame = self.current_brush[self.current_brush.len() - 1].clone();
        self = self.frame();
        for i in next_frame.iter_mut() {
            i.offset = i.offset + Point2D::new(0.0, -1.0);
        }
        self.current_brush.push(next_frame);
        self
    }

    /// Helper for regular tiles.
    pub fn tile(self, x: u32, y: u32) -> Self { self.splat(x, y, 32, 32).offset(16, 16) }

    /// Helper for blob chunks.
    ///
    /// Blobs are built from three 96x32 strips. First one contains the vertical edges, the second
    /// contains the rear blob and the third contains the blob front. The vertical and rear
    /// frames are nondescript and will probably be reused extensively.
    ///
    /// Blob shaping is somewhat complicated and requires a large number of frames.
    pub fn blob(self, vert_x: u32, vert_y: u32, rear_x: u32, rear_y: u32, x: u32, y: u32) -> Self {
        self.splat(vert_x, vert_y, 16, 32).offset(16, 16)               // 0: Top left    VERTICAL SIDES
            .frame().splat(vert_x + 16, vert_y, 16, 32).offset(0, 16)   // 1: Top right
            .frame().splat(vert_x + 32, vert_y, 16, 32).offset(16, 16)  // 2: Middle left
            .frame().splat(vert_x + 48, vert_y, 16, 32).offset(0, 16)   // 3: Middle right
            .frame().splat(vert_x + 64, vert_y, 16, 32).offset(16, 16)  // 4: Bottom left
            .frame().splat(vert_x + 80, vert_y, 16, 32).offset(0, 16)   // 5: Bottom right

            .frame().splat(rear_x, rear_y, 10, 32).offset(16, 16)       // 6: Left half       REAR PARTS

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

            .frame().splat(x, y, 10, 32).offset(16, 16)                 // 18 Left half      FRONT PARTS

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
        self.splat(center_x, center_y, 16, 32).offset(16, 16)               // 0
            .frame().splat(center_x + 16, center_y, 16, 32).offset(0, 16)   // 1
            .frame().splat(sides_x, sides_y, 16, 32).offset(16, 16)         // 2
            .frame().splat(sides_x + 16, sides_y, 16, 32).offset(0, 16)     // 3
    }

    /// Start a new frame in the current brush.
    pub fn frame(mut self) -> Self {
        self.current_brush.push(Vec::new());
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
        mem::swap(&mut brush, &mut self.current_brush);
        assert!(!brush.is_empty());

        self.brushes.insert(name, generic::Brush(brush));

        // Reset color
        self.color = WHITE;

        self
    }

    /// Construct the actual brush resources.
    ///
    /// Build an atlas using the resource construction context.
    ///
    /// Currently has the hardcoded assumption from Glium backend that the underlying texture data
    /// handle is `uint`.
    pub fn finish<F>(self, build_texture: F)
        where F: FnMut(ImageBuffer) -> usize
    {
        let mut specs: Vec<(SubImageSpec, usize)> = self.splat_images.into_iter().collect();
        specs.sort_by_key(|x| x.1);
        let specs: Vec<SubImageSpec> = specs.into_iter().map(|x| x.0).collect();
        // XXX: Really inefficient clone spam here to get the ImageBuffer set into the correct type
        // for atlas builder.
        //
        // Not terribly bad since this is only run once when initializing the resources.
        let imgs: Vec<ImageBuffer> = specs.iter()
                                          .map(|key| {
                                              let x: Resource<ImageBuffer, SubImageSpec> =
                                                  Resource::new(key.clone()).unwrap();
                                              (*x).clone()
                                          })
                                          .collect();
        let items = vitral_atlas::build(&imgs, 2048, build_texture).unwrap();

        // Now that we finally have the atlas data, use them to map our temporary brushes into the
        // final resources that will be used at runtime.
        for (name, brush) in self.brushes.iter() {
            let resource_brush: Brush = brush.clone().map(|idx| items[idx].clone());
            Brush::insert_resource(name.clone(), resource_brush);
        }
    }
}
