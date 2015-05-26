use std::default::{Default};
use std::cmp::{min, max};
use std::convert::{Into};
use image::{Primitive, GenericImage, Pixel, ImageBuffer, Rgba};
use geom::{V2, Rect};
use rgb;

/// Set alpha channel to transparent if pixels have a specific color.
pub fn color_key<P: Pixel<Subpixel=u8>, I: GenericImage<Pixel=P>, C: Into<rgb::SRgba>>(
    image: &I, color: C) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
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
pub fn crop_alpha<T: Primitive+Default, P: Pixel<Subpixel=T>, I: GenericImage<Pixel=P>>(
    image: &I) -> Rect<i32> {
    let (w, h) = image.dimensions();
    let mut p1 = V2(w as i32, h as i32);
    let mut p2 = V2(0i32, 0i32);
    let transparent: T = Default::default();
    for y in 0..(h as i32) {
        for x in 0..(w as i32) {
            let (_, _, _, a) = image.get_pixel(x as u32, y as u32).channels4();
            if a != transparent {
                p1.0 = min(x, p1.0);
                p2.0 = max(x + 1, p2.0);
                p1.1 = min(y, p1.1);
                p2.1 = max(y + 1, p2.1);
            }
        }
    }

    if p1.0 > p2.0 { Rect(V2(0, 0), V2(0, 0)) } // Empty image.
    else { Rect(p1, p2 - p1) }
}

pub fn blit<T, P, I, J>(image: &I, target: &mut J, offset: V2<i32>)
    where T: Primitive+Default,
          P: Pixel<Subpixel=T>,
          I: GenericImage<Pixel=P>,
          J: GenericImage<Pixel=P> {
    let (w, h) = image.dimensions();
    // TODO: Check for going over bounds.
    for y in 0..(h) {
        for x in 0..(w) {
            target.put_pixel(x + offset.0 as u32, y + offset.1 as u32, image.get_pixel(x, y));
        }
    }
}
