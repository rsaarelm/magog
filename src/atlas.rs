use std::iter;
use std::cmp::{max};
use num::{Float, Zero};
use num::traits::{Num};
use image::{GenericImage, ImageBuffer, Rgba, Pixel};
use img;
use geom::{V2, Rect};

/// Constructor object for atlases.
pub struct AtlasBuilder {
    images: Vec<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    draw_offsets: Vec<V2<i32>>,
}

impl AtlasBuilder {
    pub fn new() -> AtlasBuilder {
        AtlasBuilder {
            images: vec![],
            draw_offsets: vec![],
        }
    }

    /// Add an image to the image atlas with the given draw offset.
    pub fn push<P: Pixel<Subpixel=u8> + 'static, I: GenericImage<Pixel=P>>(
        &mut self, offset: V2<i32>, image: &I) -> usize {
        let Rect(pos, dim) = img::crop_alpha(image);
        let image = ImageBuffer::from_fn(
            dim.0 as u32, dim.1 as u32,
            |x, y| image.get_pixel(
                pos.0 as u32 + x, pos.1 as u32 + y).to_rgba());
        self.images.push(image);
        self.draw_offsets.push(pos + offset);
        self.images.len() - 1
    }
}

/// A collection of images packed into a single large image for more efficient
/// caching in graphics hardware.
pub struct Atlas {
    /// The atlas image containing all the subimages.
    pub image: ImageBuffer<Rgba<u8>, Vec<u8>>,
    /// Metadata for the subimages.
    pub items: Vec<AtlasItem>,
}

/// One image stored in a texture atlas
#[derive(Copy, Clone, Debug)]
pub struct AtlasItem {
    /// Vertices for the image rectangle when drawn at origin
    pub pos: Rect<f32>,
    /// Texture coordinates for the image rectangle on the atlas texture
    pub tex: Rect<f32>,
}

impl Atlas {
    pub fn new(builder: &AtlasBuilder) -> Atlas {
        let dims : Vec<V2<i32>> = builder.images.iter()
            .map(|img| { let (w, h) = img.dimensions(); V2(w as i32, h as i32) })
            .collect();

        // Add 1 pixel edges to images to prevent texturing artifacts from
        // adjacent pixels in separate subimages.
        let expanded_dims = dims.iter()
            .map(|&v| v + V2(1, 1))
            .collect();

        // Guesstimate the size for the atlas container.
        let total_area = dims.iter().map(|dim| dim.0 * dim.1).fold(0, |a, b| a + b);
        let mut d = ((total_area as f64).sqrt() as u32).next_power_of_two();
        let mut offsets;

        loop {
            assert!(d < 1000000000); // Sanity check
            match pack_rectangles(V2(d as i32, d as i32), &expanded_dims) {
                Some(ret) => {
                    offsets = ret;
                    break;
                }
                None => {
                    d = d * 2;
                }
            }
        }

        // Blit subimages to atlas image.
        let mut image: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(d, d);
        for (i, &offset) in offsets.iter().enumerate() {
            img::blit(&builder.images[i], &mut image, offset);
        }

        let image_dim = V2(d, d);

        assert!(offsets.len() == builder.draw_offsets.len());

        let items: Vec<AtlasItem> = (0..(offsets.len()))
            .map(|i| AtlasItem {
                pos: Rect(builder.draw_offsets[i].map(|x| x as f32), dims[i].map(|x| x as f32)),
                tex: Rect(scale_vec(offsets[i], image_dim), scale_vec(dims[i], image_dim))
            })
            .collect();

        return Atlas {
            image: image,
            items: items,
        };

        fn scale_vec(pixel_vec: V2<i32>, image_dim: V2<u32>) -> V2<f32> {
            V2(pixel_vec.0 as f32 / image_dim.0 as f32,
              pixel_vec.1 as f32 / image_dim.1 as f32)
        }
    }
}

/// Try to pack several small rectangles into one large rectangle. Return
/// offsets for the subrectangles within the container if a packing was found.
fn pack_rectangles<T: Num+PartialOrd+Ord+Copy>(
    container_dim: V2<T>,
    dims: &Vec<V2<T>>)
    -> Option<Vec<V2<T>>> {
    let init: T = Zero::zero();
    let total_area = dims.iter().map(|dim| dim.0 * dim.1).fold(init, |a, b| a + b);

    // Too much rectangle area to fit in container no matter how you pack it.
    // Fail early.
    if total_area > container_dim.0 * container_dim.1 { return None; }

    // Take enumeration to keep the original indices around.
    let mut largest_first : Vec<(usize, &V2<T>)> = dims.iter().enumerate().collect();
    largest_first.sort_by(|&(_i, a), &(_j, b)| (b.0 * b.1).cmp(&(a.0 * a.1)));

    let mut slots = vec![Rect(V2(Zero::zero(), Zero::zero()), container_dim)];

    let mut ret: Vec<V2<T>> = iter::repeat(V2(Zero::zero(), Zero::zero())).take(dims.len()).collect();

    for i in 0..(largest_first.len()) {
        let (idx, &dim) = largest_first[i];
        match place(dim, &mut slots) {
            Some(pos) => { ret[idx] = pos; }
            None => { return None; }
        }
    }

    return Some(ret);

    ////////////////////////////////////////////////////////////////////////

    /// Find the smallest slot in the slot vector that will fit the given
    /// item.
    fn place<T: Num+PartialOrd+Ord+Copy>(
        dim: V2<T>, slots: &mut Vec<Rect<T>>) -> Option<V2<T>> {
        for i in 0..(slots.len()) {
            let Rect(slot_pos, slot_dim) = slots[i];
            if fits(dim, slot_dim) {
                // Remove the original slot, it gets the item. Add the two new
                // rectangles that form around the item.
                let (new_1, new_2) = remaining_rects(dim, Rect(slot_pos, slot_dim));
                slots.swap_remove(i);
                slots.push(new_1);
                slots.push(new_2);
                // Sort by area from smallest to largest.
                slots.sort_by(|&a, &b| a.area().cmp(&b.area()));
                return Some(slot_pos);
            }
        }
        None
    }

    /// Return the two remaining parts of container rect when the dim-sized
    /// item is placed in the top left corner.
    fn remaining_rects<T: Num+PartialOrd+Ord+Copy>(
        dim: V2<T>, Rect(rect_pos, rect_dim): Rect<T>) ->
        (Rect<T>, Rect<T>) {
        assert!(fits(dim, rect_dim));

        // Choose between making a vertical or a horizontal split
        // based on which leaves a bigger open rectangle.
        let vert_vol = max(rect_dim.0 * (rect_dim.1 - dim.1),
            (rect_dim.0 - dim.0) * dim.1);
        let horiz_vol = max(dim.0 * (rect_dim.1 - dim.1),
            (rect_dim.0 - dim.0) * rect_dim.1);

        if vert_vol > horiz_vol {
            //     |AA
            // ----+--
            // BBBBBBB
            // BBBBBBB
            (Rect(V2(rect_pos.0 + dim.0, rect_pos.1), V2(rect_dim.0 - dim.0, dim.1)),
             Rect(V2(rect_pos.0, rect_pos.1 + dim.1), V2(rect_dim.0, rect_dim.1 - dim.1)))
        } else {
            //     |BB
            // ----+BB
            // AAAA|BB
            // AAAA|BB
            (Rect(V2(rect_pos.0, rect_pos.1 + dim.1), V2(dim.0, rect_dim.1 - dim.1)),
             Rect(V2(rect_pos.0 + dim.0, rect_pos.1), V2(rect_dim.0 - dim.0, rect_dim.1)))
        }
    }

    fn fits<T: Ord>(dim: V2<T>, container_dim: V2<T>) -> bool {
        dim.0 <= container_dim.0 && dim.1 <= container_dim.1
    }
}
