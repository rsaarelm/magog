extern crate euclid;
extern crate vitral;

use std::cmp::max;
use euclid::{Point2D, Rect, Size2D};
use vitral::ImageBuffer;

/// An incremental texture atlas.
///
/// Assumes a backend system where the resource behind the texture handle `T` can be changed
/// without changing the value of the handle itself.
pub struct Atlas<T> {
    texture: T,
    slots: Vec<Rect<u32>>,
    placed: Vec<Rect<u32>>,
    atlas: ImageBuffer,
    is_dirty: bool,
}

impl<T: Clone> Atlas<T> {
    pub fn new(texture: T, size: Size2D<u32>) -> Atlas<T> {
        Atlas {
            texture: texture,
            slots: vec![Rect::new(Point2D::new(0, 0), size)],
            placed: Vec::new(),
            atlas: ImageBuffer::new(size.width, size.height),
            is_dirty: false,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.placed.is_empty()
    }

    pub fn add(&mut self, image: &ImageBuffer) -> Option<vitral::ImageData<T>> {
        if let Some(area) = self.place(image.size) {
            // Draw the new image into the atlas image.
            self.atlas.copy_from(image, area.origin.x, area.origin.y);

            // Map texture coordinates to the unit rectangle.
            let x_scale = 1.0 / self.atlas.size.width as f32;
            let y_scale = 1.0 / self.atlas.size.height as f32;

            let tex_pos = Point2D::new(area.origin.x as f32 * x_scale,
                                       area.origin.y as f32 * y_scale);
            let tex_size = Size2D::new(area.size.width as f32 * x_scale,
                                       area.size.height as f32 * y_scale);
            Some(vitral::ImageData {
                texture: self.texture.clone(),
                size: image.size,
                tex_coords: Rect::new(tex_pos, tex_size),
            })
        } else {
            None
        }
    }

    pub fn is_dirty(&self) -> bool { self.is_dirty }

    pub fn size(&self) -> Size2D<u32> { self.atlas.size }

    pub fn texture<'a>(&'a self) -> &'a T { &self.texture }

    /// Write the current atlas image to the system texture handle if the atlas has changed.
    ///
    /// The texture update function must be provided by the caller.
    pub fn update_texture<F>(&mut self, mut f: F)
        where F: FnMut(&ImageBuffer, &T)
    {
        if self.is_dirty {
            f(&self.atlas, &self.texture);
            self.is_dirty = false;
        }
    }

    /// Find the smallest slot in the slot vector that will fit the given item.
    ///
    /// Return `None` if the item will not fit in this atlas.
    fn place(&mut self, size: Size2D<u32>) -> Option<Rect<u32>> {
        for i in 0..self.slots.len() {
            let Rect { origin: slot_pos, size: slot_dim } = self.slots[i];
            if fits(size, slot_dim) {
                // Remove the original slot, it gets the item. Add the two new
                // rectangles that form around the item.
                let (new_1, new_2) = remaining_rects(size, self.slots[i]);
                self.slots.swap_remove(i);
                self.slots.push(new_1);
                self.slots.push(new_2);
                // Sort by area from smallest to largest.
                self.slots.sort_by(|&a, &b| {
                    (a.size.width * a.size.height).cmp(&(b.size.width * b.size.height))
                });
                let ret = Rect::new(slot_pos, size);
                self.placed.push(ret);
                self.is_dirty = true;
                return Some(ret);
            }
        }
        return None;

        fn fits(size: Size2D<u32>, container_dim: Size2D<u32>) -> bool {
            size.width <= container_dim.width && size.height <= container_dim.height
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
                           Size2D::new(rect_dim.width, rect_dim.height - dim.height)))
            } else {
                //     |BB
                // ----+BB
                // AAAA|BB
                // AAAA|BB
                (Rect::new(Point2D::new(rect_pos.x, rect_pos.y + dim.height),
                           Size2D::new(dim.width, rect_dim.height - dim.height)),
                 Rect::new(Point2D::new(rect_pos.x + dim.width, rect_pos.y),
                           Size2D::new(rect_dim.width - dim.width, rect_dim.height)))
            }
        }
    }
}
