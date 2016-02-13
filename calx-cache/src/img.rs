use std::cmp::{min, max};
use std::convert::Into;
use num::traits::Zero;
use image::{GenericImage, Pixel, ImageBuffer, SubImage, Rgba};
use calx_layout::{Anchor, Rect, Shape2D};
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
            pa = Zero::zero()
        }
        Pixel::from_channels(pr, pg, pb, pa)
    })
}

/// Return the rectangle enclosing the parts of the image that aren't fully
/// transparent.
pub fn crop_alpha<I>(image: &I) -> Rect<i32>
    where I: GenericImage
{
    let (w, h) = image.dimensions();
    let (mut x1, mut y1) = (w as i32, h as i32);
    let (mut x2, mut y2) = (0i32, 0i32);
    let transparent: <I::Pixel as Pixel>::Subpixel = Zero::zero();
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

pub fn blit<P, I, J, V>(image: &I, target: &mut J, offset: V)
    where P: Pixel,
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

/// Convenience function for extracting a subimage with a Rect as bounds.
#[inline(always)]
pub fn subimage<'a, I>(image: &'a mut I, rect: &Rect<i32>) -> SubImage<'a, I>
    where I: GenericImage + 'static,
          I::Pixel: 'static,
          <I::Pixel as Pixel>::Subpixel: 'static
{
    image.sub_image(rect.top[0] as u32,
                    rect.top[1] as u32,
                    rect.size[0] as u32,
                    rect.size[1] as u32)
}

/// Return the tiles on a tile sheet image.
///
/// Tiles are bounding boxes of non-background pixel groups surrounded by
/// only background pixels or image edges. Background color is the color of
/// the bottom right corner pixel of the image. The bounding boxes are
/// returned lexically sorted by the coordinates of their bottom right
/// corners, first along the y-axis then along the x-axis. This produces a
/// natural left-to-right, bottom-to-top ordering for a cleanly laid out
/// tile sheet.
pub fn tilesheet_bounds<I>(image: &I) -> Vec<Rect<i32>>
    where I: GenericImage,
          I::Pixel: PartialEq
{
    let mut ret = Vec::new();
    let image_rect = Rect::new([0, 0],
                               [image.width() as i32, image.height() as i32]);
    let background = image.get_pixel(image.width() - 1, image.height() - 1);

    for pt in image_rect.tiles([1, 1]).map(|x| x.top) {
        // Skip areas that already contain known tiles.
        //
        // XXX: Using the union Shape2D is an ineffective way to test against
        // the area to skip.
        if ret.contains(pt) {
            continue;
        }

        if image.get_pixel(pt[0] as u32, pt[1] as u32) != background {
            ret.push(tile_bounds(image, pt, background));
        }
    }

    ret.sort_by(|a, b| rect_key(a).cmp(&rect_key(b)));
    return ret;

    fn rect_key(x: &Rect<i32>) -> (i32, i32) {
        let p = x.point(Anchor::BottomRight);
        (p[1], p[0])
    }
}

/// Find the smallest bounding box around seed pixel whose sides are either
/// all background color or image edge.
fn tile_bounds<I>(image: &I,
                  seed_pos: [i32; 2],
                  background: I::Pixel)
                  -> Rect<i32>
    where I: GenericImage,
          I::Pixel: PartialEq
{
    let image_rect = Rect::new([0, 0],
                               [image.width() as i32, image.height() as i32]);
    let mut ret = Rect::new(seed_pos, [seed_pos[0] + 1, seed_pos[1] + 1]);

    struct Edge {
        p1: Anchor,
        p2: Anchor,
        d: [i32; 2],
    };

    const EDGE: [Edge; 4] = [Edge {
                                 p1: Anchor::TopLeft,
                                 p2: Anchor::TopRight,
                                 d: [0, -1],
                             },
                             Edge {
                                 p1: Anchor::TopRight,
                                 p2: Anchor::BottomRight,
                                 d: [1, 0],
                             },
                             Edge {
                                 p1: Anchor::BottomRight,
                                 p2: Anchor::BottomLeft,
                                 d: [0, 1],
                             },
                             Edge {
                                 p1: Anchor::BottomLeft,
                                 p2: Anchor::TopLeft,
                                 d: [-1, 0],
                             }];

    let mut unchanged_count = 0;

    'outer: loop {
        for dir in 0..4 {
            if unchanged_count >= 4 {
                break 'outer;
            }

            let p1 = ret.point(EDGE[dir].p1);
            let mut p2 = ret.point(EDGE[dir].p2);
            p2[0] += EDGE[dir].d[0];
            p2[1] += EDGE[dir].d[1];
            let expand = Rect::new(p1, p2);

            // Expansion would go outside the image area, abandon expansion.
            if !image_rect.contains_rect(&expand) {
                unchanged_count += 1;
                continue;
            }

            // Expansion has only background pixels, abandon expansion.
            if expand.tiles([1, 1])
                     .map(|x| x.top)
                     .all(|p| {
                         image.get_pixel(p[0] as u32, p[1] as u32) == background
                     }) {
                unchanged_count += 1;
                continue;
            }

            // Otherwise add the expansion to the result.
            ret = vec![ret, expand].bounding_box();
            unchanged_count = 0;
        }
    }

    ret
}

/// Interface for objects that store multiple images, like an image atlas.
pub trait ImageStore<H>: Sized {
    fn add_image<I, V, P>(&mut self, offset: V, image: &I) -> H
        where I: GenericImage<Pixel = P>,
              P: Pixel<Subpixel = u8>,
              V: Into<[i32; 2]>;

    /// Add a single-pixel solid image to the store to be used with solid
    /// color shapes.
    ///
    /// You may want to call this as the first thing with a new store
    /// (assuming the image handles are something like numbers couting up from
    /// 0), to get the default image handle to point to the solid texture.
    fn add_solid_image(&mut self) -> H {
        let image: ImageBuffer<Rgba<u8>, Vec<u8>>;
        image = ImageBuffer::from_fn(1, 1, |_, _| {
            Rgba([0xffu8, 0xffu8, 0xffu8, 0xffu8])
        });
        self.add_image([0, 0], &image)
    }
}
