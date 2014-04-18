extern crate cgmath;

use std::vec::Vec;
use std::num::zero;
use cgmath::aabb::{Aabb, Aabb2};
use cgmath::vector::Vector2;
use cgmath::partial_ord::PartOrdPrim;
use cgmath::point::{Point, Point2};

fn fits<S: Primitive + Num + PartOrdPrim>(dim: &Vector2<S>, rect: &Aabb2<S>) -> bool {
    let rect_dim = rect.dim();
    dim.x <= rect_dim.x && dim.y <= rect_dim.y
}

fn pack_into<S: Primitive + PartOrdPrim>(dim: &Vector2<S>, rect: &Aabb2<S>) ->
    (Aabb2<S>, (Aabb2<S>, Aabb2<S>)) {
    assert!(fits(dim, rect));

    let fit = Aabb2::new(rect.min().clone(), rect.min().add_v(dim));
    let rect_dim = rect.dim();

    // Choose between making a vertical or a horizontal split
    // based on which leaves a bigger open rectangle.
    let vert_vol = (rect_dim.x * (rect_dim.y - dim.y)).max(
        (rect_dim.x - dim.x) * dim.y);
    let horiz_vol = (dim.x * (rect_dim.y - dim.y)).max(
        (rect_dim.x - dim.x) * rect_dim.y);

    if vert_vol > horiz_vol {
        // fit |AA
        // ----+--
        // BBBBBBB
        // BBBBBBB
        (fit, (Aabb2::new(
                rect.min().add_v(&Vector2::new(dim.x.clone(), zero::<S>())),
                rect.min().add_v(&Vector2::new(rect_dim.x, dim.y.clone()))),
            Aabb2::new(
                rect.min().add_v(&Vector2::new(zero::<S>(), dim.y.clone())),
                rect.max().clone())))
    } else {
        // fit |BB
        // ----+BB
        // AAAA|BB
        // AAAA|BB
        (fit, (Aabb2::new(
                rect.min().add_v(&Vector2::new(zero::<S>(), dim.y.clone())),
                rect.min().add_v(&Vector2::new(dim.x.clone(), rect_dim.y))),
            Aabb2::new(
                rect.min().add_v(&Vector2::new(dim.x.clone(), zero::<S>())),
                rect.max().clone())))
    }
}

struct Packing<S> {
    // Invariant: Slots are kept sorted from smallest to largest.
    slots: Vec<Aabb2<S>>,
}

impl<S: Primitive + TotalOrd + PartOrdPrim> Packing<S> {
    pub fn new(area: &Aabb2<S>) -> Packing<S> {
        Packing{slots: vec!(area.clone())}
    }

    pub fn fit(&mut self, dim: &Vector2<S>) -> Option<Aabb2<S>> {
        for i in range(0, self.slots.len()) {
            if fits(dim, self.slots.get(i)) {
                let (ret, (new_1, new_2)) = pack_into(dim, self.slots.get(i));
                self.slots.swap_remove(i);
                self.slots.push(new_1);
                self.slots.push(new_2);
                self.slots.sort_by(|a, b| (a.volume()).cmp(&(b.volume())));
                return Some(ret);
            }
        }
        None
    }
}

pub fn try_pack_rects<S: Primitive + TotalOrd + PartOrdPrim>(
    rect: &Aabb2<S>, sizes: &[Vector2<S>]) -> Option<Vec<Aabb2<S>>> {
    // Store the original indices so that we know what was where even if we
    // change the order.
    let mut indexed : Vec<(uint, &Vector2<S>)> = sizes.iter().enumerate().collect();
    // Sort the list from the size denoting the largest area down.
    indexed.sort_by(|&(_i, v1), &(_j, v2)| (v2.x * v2.y).cmp(&(v1.x * v1.y)));
    let mut packer = Packing::new(rect);
    let mut ret = Vec::from_elem(sizes.len(), Aabb2::new(
            Point2::new(zero(), zero()),
            Point2::new(zero(), zero())));

    for &(i, d) in indexed.iter() {
        match packer.fit(d) {
            Some(rect) => ret.as_mut_slice()[i] = rect,
            None => return None
        }
    }
    Some(ret)
}

pub fn pack_rects<S: Primitive + TotalOrd + PartOrdPrim>(
    rect: &Aabb2<S>, sizes: &[Vector2<S>]) -> (Aabb2<S>, Vec<Aabb2<S>>) {
    let next_rect = Aabb2::new(
        rect.min().clone(), rect.min().add_v(&(rect.dim() + rect.dim())));
    match try_pack_rects(rect, sizes) {
        Some(ret) => (rect.clone(), ret),
        None => pack_rects(&next_rect, sizes)
    }
}
