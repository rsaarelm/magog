use std::default::Default;
use std::cmp::{min, max};
use std::convert::Into;
use image::{Primitive, GenericImage, Pixel, ImageBuffer, Rgba};
use calx_layout::Rect;
use calx_color;

/// Set alpha channel to transparent if pixels have a specific color.
pub fn color_key<P, I, C>(image: &I, color: C) -> ImageBuffer<Rgba<u8>, Vec<u8>>
    where P: Pixel<Subpixel = u8>,
          I: GenericImage<Pixel = P>,
          C: Into<calx_color::SRgba>
{
    let (w, h) = image.dimensions();
    let srgba = color.into();
    ImageBuffer::from_fn(w, h, |x, y| {
        let (pr, pg, pb, mut pa) = image.get_pixel(x, y).to_rgba().channels4();
        if pr == srgba.r && pg == srgba.g && pb == srgba.b {
            pa = Default::default();
        }
        Pixel::from_channels(pr, pg, pb, pa)
    })
}

/// Return the rectangle enclosing the parts of the image that aren't fully
/// transparent.
pub fn crop_alpha<T, P, I>(image: &I) -> Rect<i32>
    where T: Primitive + Default,
          P: Pixel<Subpixel = T>,
          I: GenericImage<Pixel = P>
{
    let (w, h) = image.dimensions();
    let (mut x1, mut y1) = (w as i32, h as i32);
    let (mut x2, mut y2) = (0i32, 0i32);
    let transparent: T = Default::default();
    for y in 0..(h as i32) {
        for x in 0..(w as i32) {
            let (_, _, _, a) = image.get_pixel(x as u32, y as u32).channels4();
            if a != transparent {
                x1 = min(x, x1);
                x2 = max(x + 1, x2);
                y1 = min(y, y1);
                y2 = max(y + 1, y2);
            }
        }
    }

    if x1 > x2 {
        Rect::new([0, 0], [0, 0])
    } else {
        Rect::new([x1, y1], [x2, y2])
    }
}

pub fn blit<T, P, I, J, V>(image: &I, target: &mut J, offset: V)
    where T: Primitive + Default,
          P: Pixel<Subpixel = T>,
          I: GenericImage<Pixel = P>,
          J: GenericImage<Pixel = P>,
          V: Into<[i32; 2]>
{
    let (w, h) = image.dimensions();
    let offset = offset.into();
    // TODO: Check for going over bounds.
    for y in 0..(h) {
        for x in 0..(w) {
            target.put_pixel(x + offset[0] as u32,
                             y + offset[1] as u32,
                             image.get_pixel(x, y));
        }
    }
}

/// Interface for objects that store multiple images, like an image atlas.
pub trait ImageStore<H>: Sized {
    fn add_image<P, I, V>(&mut self, offset: V, image: &I) -> H
        where P: Pixel<Subpixel = u8> + 'static,
              I: GenericImage<Pixel = P>,
              V: Into<[i32; 2]>;
}
