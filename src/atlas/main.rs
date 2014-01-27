extern mod stb;
extern mod cgmath;

use std::vec;
use std::io::File;
use std::num::{max};
use std::iter::AdditiveIterator;

use stb::image::Image;

use cgmath::vector::Vec2;
use cgmath::point::Point2;

// TODO: Get cgmath Aabb2 to have rectangle-like structure, use it instead of
// rect

// TODO Move rect packing into shared library.
#[deriving(Clone, Eq)]
struct Rect<S> {
    pos: Point2<S>,
    size: Vec2<S>,
}

impl<S> Rect<S> {
    #[inline]
    pub fn new(x: S, y: S, w: S, h: S) -> Rect<S> {
        Rect{
            pos: Point2{x: x, y: y},
            size: Vec2{x: w, y: h}
        }
    }
}

fn dim_fits(dim: Vec2<int>, rect: Rect<int>) -> bool {
    dim.x <= rect.size.x && dim.y <= rect.size.y
}

fn pack_into(dim: Vec2<int>, rect: Rect<int>) ->
    (Rect<int>, (Rect<int>, Rect<int>)) {
    assert!(dim_fits(dim, rect));

    let fit = Rect{pos: rect.pos, size: dim};

    // Choose between making a vertical or a horizontal split
    // based on which leaves a bigger open rectangle.
    let vert_vol = max(
        rect.size.x * (rect.size.y - dim.y),
        (rect.size.x - dim.x) * dim.y);
    let horiz_vol = max(
        dim.x * (rect.size.y - dim.y),
        (rect.size.x - dim.x) * rect.size.y);

    if vert_vol > horiz_vol {
        // fit |AA
        // ----+--
        // BBBBBBB
        // BBBBBBB
        (fit, (Rect::new(rect.pos.x + dim.x, rect.pos.y,
                rect.size.x - dim.x, dim.y),
            Rect::new(
                rect.pos.x, rect.pos.y + dim.y,
                rect.size.x, rect.size.y - dim.y)))
           /* 
            Rect{
                pos: Point2{x: rect.pos.x, y: rect.pos.y + dim.y},
                size: Vec2{x: rect.size.x, y: rect.size.y - dim.y}}))
                */
    } else {
        // fit |BB
        // ----+BB
        // AAAA|BB
        // AAAA|BB
        (fit, (Rect{
                pos: Point2{x: rect.pos.x, y: rect.pos.y + dim.y},
                size: Vec2{x: dim.x, y: rect.size.y - dim.y}},
            Rect{
                pos: Point2{x: rect.pos.x + dim.x, y: rect.pos.y},
                size: Vec2{x: rect.size.x - dim.x, y: rect.size.y}}))
    }
}

struct Packing {
    // Invariant: Slots are kept sorted from smallest to largest.
    slots: ~[Rect<int>],
}

impl Packing {
    pub fn new(area: Rect<int>) -> Packing {
        Packing{slots: ~[area]}
    }

    pub fn fit(&mut self, dim: Vec2<int>) -> Option<Rect<int>> {
        for i in range(0, self.slots.len()) {
            if dim_fits(dim, self.slots[i]) {
                let (ret, (new_1, new_2)) = pack_into(dim, self.slots[i]);
                self.slots.remove(i);
                self.slots.push(new_1);
                self.slots.push(new_2);
                self.slots.sort_by(|a, b| (a.size.x * a.size.y).cmp(&(b.size.x * b.size.y)));
                return Some(ret);
            }
        }
        None
    }
}

// TODO: Sort inputs from largest to smallest in pack_rects before adding them.
// TODO: Add an uint parameter to specify edge size.
pub fn pack_rects(base_dim: int, dims: &[Vec2<int>]) -> (Rect<int>, ~[Rect<int>]) {
    // Something's probably exploding if we hit this.
    assert!(base_dim <= 1000000000);
    let area = Rect{pos: Point2{x: 0, y: 0}, size: Vec2{x: base_dim, y: base_dim}};
    let mut packer = Packing::new(area);
    let mut poses = ~[];
    for i in dims.iter() {
        match packer.fit(*i) {
            Some(rect) => poses.push(rect),
            None => // Looks like our base was too small, try with a bigger one.
                return pack_rects(base_dim * 2, dims)
        }
    }

    (area, poses)
}

pub fn guesstimate_dim(dims: &[Vec2<int>]) -> int {
    let total_volume : int = dims.iter().map(|&v| v.x * v.y).sum();
    let mut dim = 4;
    // Find a power of two dimension that will probably fit the rects.
    // XXX: Obvious failure modes with perverse rect shapes.
    while dim * dim < total_volume {
        dim *= 2;
    }
    dim
}

pub fn main() {
    let font = stb::truetype::Font::new(
        File::open(&Path::new("assets/pf_tempesta_seven_extended_bold.ttf"))
        .read_to_end())
        .unwrap();

    let mut glyphs = vec::build(None, |push| {
        for i in range(32, 256) {
            match font.glyph(i, 13.0) {
                Some(g) => push((i, g)),
                None => ()
            }
        }
    });

    // Sort them from largest to smallest, that way they'll make for the best
    // addition order.
    glyphs.sort_by(|&(ref _i1, ref g1), &(ref _i2, ref g2)|
                   (g2.width * g2.height).cmp(&(g1.width * g1.height)));

    // Add gaps to counter texture artifacts from fonts sticking to each other.
    let dims = glyphs.map(|&(ref _i, ref g)| Vec2::new(g.width + 1, g.height + 1));
    let (base, pack) = pack_rects(guesstimate_dim(dims), dims);

    let mut img = Image::new(base.size.x as uint, base.size.y as uint, 1);
    let w = base.size.x;

    for i in range(0, glyphs.len()) {
        let (ref _i, ref g) = glyphs[i];
        let rect = pack[i];
        for y in range(0, g.height) {
            for x in range(0, g.width) {
                img.pixels[(x + rect.pos.x) + (y + rect.pos.y) * w] = g.pixels[x + g.width * y]
            }
        }
    }

    img.save_png("/tmp/font.png");

    println!("{} {}", base.size.x, base.size.y);
}
