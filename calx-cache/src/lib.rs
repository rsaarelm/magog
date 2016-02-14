extern crate num;
extern crate vec_map;
extern crate image;
extern crate calx_layout;
extern crate calx_color;

pub use atlas::{AtlasBuilder, Atlas, AtlasItem};
pub use font::{Glyph, Font};
pub use img::{color_key, subimage, tilesheet_bounds};
pub use index_cache::{IndexCache, CacheKey};

mod atlas;
mod brush;
mod font;
mod img;
mod index_cache;

/// Interface for objects that store multiple images, like an image atlas.
pub trait ImageStore: Sized {
    /// Add an image to the store.
    ///
    /// Return a handle value that can be used to access the stored image
    /// later.
    fn add_image<I, V, P>(&mut self, center: V, image: &I) -> usize
        where I: image::GenericImage<Pixel = P>,
              P: image::Pixel<Subpixel = u8>,
              V: Into<[i32; 2]>;

    /// Add a single-pixel solid image to the store to be used with solid
    /// color shapes.
    ///
    /// You may want to call this as the first thing with a new store
    /// to get the default (0) image handle to point to the solid texture.
    fn add_solid_image(&mut self) -> usize {
        let image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;
        image = image::ImageBuffer::from_fn(1, 1, |_, _| {
            image::Rgba([0xffu8, 0xffu8, 0xffu8, 0xffu8])
        });
        self.add_image([0, 0], &image)
    }
}
