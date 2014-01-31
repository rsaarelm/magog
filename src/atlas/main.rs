extern mod stb;
extern mod cgmath;
extern mod calx;

use std::vec;
use std::io::File;
use std::iter::AdditiveIterator;

use stb::image::Image;

use calx::pack_rect::pack_rects;

use cgmath::aabb::{Aabb, Aabb2};
use cgmath::point::Point2;
use cgmath::vector::Vec2;

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

    let glyphs = vec::build(None, |push| {
        for i in range(32, 256) {
            match font.glyph(i, 13.0) {
                Some(g) => push((i, g)),
                None => ()
            }
        }
    });

    // Add gaps to counter texture artifacts from fonts sticking to each other.
    let dims = glyphs.map(|&(ref _i, ref g)| Vec2::new(g.width + 1, g.height + 1));

    let size = guesstimate_dim(dims);
    let base = Aabb2::new(&Point2::new(0, 0), &Point2::new(size, size));
    let (base, pack) = pack_rects(&base, dims);

    let mut img = Image::new(base.dim().x as uint, base.dim().y as uint, 1);
    let w = base.dim().x;

    for i in range(0, glyphs.len()) {
        let (ref _i, ref g) = glyphs[i];
        let rect = pack[i];
        for y in range(0, g.height) {
            for x in range(0, g.width) {
                img.pixels[(x + rect.min().x) + (y + rect.min().y) * w] = g.pixels[x + g.width * y]
            }
        }
    }

    img.save_png("/tmp/font.png");

    println!("{} {}", base.dim().x, base.dim().y);
}
