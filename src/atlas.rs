use std::iter;
use std::cmp::max;
use image::GenericImage;
use euclid::{Point2D, Rect, Size2D};
use ImageBuffer;

pub fn build_atlas(input: &[ImageBuffer],
                   mut max_size: u32)
                   -> Option<(ImageBuffer, Vec<Point2D<u32>>)> {
    assert!(input.len() > 0);
    let dims: Vec<Size2D<u32>> = input.iter()
                                      .map(|i| {
                                          Size2D::new(i.width(), i.height())
                                      })
                                      .collect();

    let mut packing = None;
    // Keep shrinking the atlas size until things no longer pack.
    while let Some(p) = pack_rectangles(Size2D::new(max_size, max_size),
                                        &dims) {
        packing = Some(p);
        max_size /= 2;
    }

    // Undo last scaling step.
    max_size *= 2;
    if max_size == 0 {
        max_size = 1;
    }

    if let Some(packing) = packing {
        let mut atlas = ImageBuffer::new(max_size, max_size);

        for (i, pos) in packing.iter().enumerate() {
            atlas.copy_from(&input[i], pos.x, pos.y);
        }

        Some((atlas, packing))
    } else {
        // Didn't get a packing even at the largest size, bail out.
        None
    }

}

/// Try to pack several small rectangles into one large rectangle. Return
/// offsets for the subrectangles within the container if a packing was found.
fn pack_rectangles(container_dim: Size2D<u32>,
                   dims: &Vec<Size2D<u32>>)
                   -> Option<Vec<Point2D<u32>>> {
    let total_area = dims.iter()
                         .map(|dim| dim.width * dim.height)
                         .fold(0, |a, b| a + b);

    // Too much rectangle area to fit in container no matter how you pack it.
    // Fail early.
    if total_area > container_dim.width * container_dim.height {
        return None;
    }

    // Take enumeration to keep the original indices around.
    let mut largest_first: Vec<(usize, &Size2D<u32>)> = dims.iter()
                                                            .enumerate()
                                                            .collect();
    largest_first.sort_by(|&(_i, a), &(_j, b)| {
        (b.width * b.height).cmp(&(a.width * a.height))
    });

    let mut slots = vec![Rect::new(Point2D::new(0, 0), container_dim)];

    let mut ret: Vec<Point2D<u32>> = iter::repeat(Point2D::new(0, 0))
                                         .take(dims.len())
                                         .collect();

    for i in 0..(largest_first.len()) {
        let (idx, &dim) = largest_first[i];
        match place(dim, &mut slots) {
            Some(pos) => {
                ret[idx] = pos;
            }
            None => {
                return None;
            }
        }
    }

    return Some(ret);

    /// Find the smallest slot in the slot vector that will fit the given
    /// item.
    fn place(dim: Size2D<u32>,
             slots: &mut Vec<Rect<u32>>)
             -> Option<Point2D<u32>> {
        for i in 0..(slots.len()) {
            let Rect { origin: slot_pos, size: slot_dim } = slots[i];
            if fits(dim, slot_dim) {
                // Remove the original slot, it gets the item. Add the two new
                // rectangles that form around the item.
                let (new_1, new_2) = remaining_rects(dim,
                                                     Rect::new(slot_pos,
                                                               slot_dim));
                slots.swap_remove(i);
                slots.push(new_1);
                slots.push(new_2);
                // Sort by area from smallest to largest.
                slots.sort_by(|&a, &b| {
                    (a.size.width * a.size.height)
                        .cmp(&(b.size.width * b.size.height))
                });
                return Some(slot_pos);
            }
        }
        None
    }

    /// Return the two remaining parts of container rect when the dim-sized
    /// item is placed in the top left corner.
    fn remaining_rects(dim: Size2D<u32>,
                       Rect { origin: rect_pos, size: rect_dim }: Rect<u32>)
                       -> (Rect<u32>, Rect<u32>) {
        assert!(fits(dim, rect_dim));

        // Choose between making a vertical or a horizontal split
        // based on which leaves a bigger open rectangle.
        let vert_vol = max(rect_dim.width * (rect_dim.height - dim.height),
                           (rect_dim.width - dim.width) * dim.height);
        let horiz_vol = max(dim.width * (rect_dim.height - dim.height),
                            (rect_dim.width - dim.width) * rect_dim.height);

        if vert_vol > horiz_vol {
            //     |AA
            // ----+--
            // BBBBBBB
            // BBBBBBB
            (Rect::new(Point2D::new(rect_pos.x + dim.width, rect_pos.y),
                       Size2D::new(rect_dim.width - dim.width, dim.height)),
             Rect::new(Point2D::new(rect_pos.x, rect_pos.y + dim.height),
                       Size2D::new(rect_dim.width,
                                   rect_dim.height - dim.height)))
        } else {
            //     |BB
            // ----+BB
            // AAAA|BB
            // AAAA|BB
            (Rect::new(Point2D::new(rect_pos.x, rect_pos.y + dim.height),
                       Size2D::new(dim.width, rect_dim.height - dim.height)),
             Rect::new(Point2D::new(rect_pos.x + dim.width, rect_pos.y),
                       Size2D::new(rect_dim.width - dim.width,
                                   rect_dim.height)))
        }
    }

    fn fits(dim: Size2D<u32>, container_dim: Size2D<u32>) -> bool {
        dim.width <= container_dim.width && dim.height <= container_dim.height
    }
}
