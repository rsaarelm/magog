use std::default::{Default};
use std::cmp::{min, max};
use std::num::{NumCast};
use image::{GenericImage, Pixel, ImageBuffer, Rgba};
use geom::{V2, Rect};
use rgb::Rgb;
use primitive::Primitive;

/// Set alpha channel to transparent if pixels have a specific color.
pub fn color_key<P: Pixel<u8>, I: GenericImage<P>>(
    image: &I, color: &Rgb) -> ImageBuffer<Vec<u8>, u8, Rgba<u8>> {
    let (w, h) = image.dimensions();
    ImageBuffer::from_fn(w, h, |x, y| {
        let (pr, pg, pb, mut pa) = image.get_pixel(x, y).to_rgba().channels4();
        if pr == color.r && pg == color.g && pb == color.b {
            pa = Default::default();
        }
        Pixel::from_channels(pr, pg, pb, pa)
    })
}

/// Return the rectangle enclosing the parts of the image that aren't fully
/// transparent.
pub fn crop_alpha<T: Primitive+Default, P: Pixel<T>, I: GenericImage<P>>(
    image: &I) -> Rect<i32> {
    let (w, h) = image.dimensions();
    let mut p1 = V2(w as i32, h as i32);
    let mut p2 = V2(0i32, 0i32);
    let transparent: T = Default::default();
    for y in range(0, h as i32) {
        for x in range(0, w as i32) {
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

pub fn blit<T: Primitive+Default, P: Pixel<T>, I: GenericImage<P>, J: GenericImage<P>> (
    image: &I, target: &mut J, offset: V2<i32>) {
    let (w, h) = image.dimensions();
    // TODO: Check for going over bounds.
    for y in range(0, h) {
        for x in range(0, w) {
            target.put_pixel(x + offset.0 as u32, y + offset.1 as u32, image.get_pixel(x, y));
        }
    }
}

/// Try to pack several small rectangles into one large rectangle. Return
/// offsets for the subrectangles within the container if a packing was found.
pub fn pack_rectangles<T: Primitive+Ord+Clone>(
    container_dim: V2<T>,
    dims: &Vec<V2<T>>)
    -> Option<Vec<V2<T>>> {
    let init: T = NumCast::from(0i32).unwrap();
    let total_area = dims.iter().map(|dim| dim.0 * dim.1).fold(init, |a, b| a + b);

    // Too much rectangle area to fit in container no matter how you pack it.
    // Fail early.
    if total_area > container_dim.0 * container_dim.1 { return None; }

    // Take enumeration to keep the original indices around.
    let mut largest_first : Vec<(usize, &V2<T>)> = dims.iter().enumerate().collect();
    largest_first.sort_by(|&(_i, a), &(_j, b)| (b.0 * b.1).cmp(&(a.0 * a.1)));

    let mut slots = vec![Rect(V2(NumCast::from(0i32).unwrap(), NumCast::from(0i32).unwrap()), container_dim)];

    let mut ret: Vec<V2<i32>> = dims.len().repeat(V2(NumCast::from(0i32).unwrap(), NumCast::from(0i32).unwrap())).collect();

    for i in range(0, largest_first.len()) {
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
    fn place<T: Primitive+Ord>(
        dim: V2<T>, slots: &mut Vec<Rect<T>>) -> Option<V2<T>> {
        for i in range(0, slots.len()) {
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
    fn remaining_rects<T: Primitive+Ord>(
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
