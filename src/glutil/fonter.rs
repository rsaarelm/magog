use std::vec;
use std::char;
use std::iter::AdditiveIterator;

use stb::truetype::Font;

use cgmath::aabb::{Aabb, Aabb2};
use cgmath::point::Point2;
use cgmath::vector::Vec2;

use calx::pack_rect::pack_rects;

use texture::Texture;

pub struct Fonter<M> {
    priv lookup: ~M,
    priv texture: ~Texture,
}

pub fn guesstimate_atlas_dim(dims: &[Vec2<int>]) -> int {
    let total_volume : int = dims.iter().map(|&v| v.x * v.y).sum();
    let mut dim = 4;
    // Find a power of two dimension that will probably fit the rects.
    // XXX: Obvious failure modes with perverse rect shapes.
    while dim * dim < total_volume {
        dim *= 2;
    }
    dim
}

fn scale_rect(tex_rect: &Aabb2<int>, int_rect: &Aabb2<int>) -> Aabb2<f32> {
    let dim = tex_rect.dim();
    Aabb2::new(
        &Point2::new(
            int_rect.min().x as f32 / dim.x as f32,
            int_rect.min().y as f32 / dim.y as f32),
        &Point2::new(
            int_rect.max().x as f32 / dim.x as f32,
            int_rect.max().y as f32 / dim.y as f32))
}

impl<M: FromIterator<(char, Aabb2<f32>)> + Map<char, Aabb2<f32>>> Fonter<M> {
    pub fn new(font: &Font, height: f64, start_char: uint, num_chars: uint) -> Fonter<M> {
        let glyphs = vec::build(None, |push| {
            for c in range(start_char, start_char + num_chars) {
                match font.glyph(c, height) {
                    Some(glyph) => push(glyph),
                    None => fail!("Font does not contain {}", c)
                }
            }
        });

        // Add gaps to counter texture artifacts from fonts sticking to each other.
        let dims = glyphs.map(|g| Vec2::new(g.width + 1, g.height + 1));

        let size = guesstimate_atlas_dim(dims);
        let base = Aabb2::new(&Point2::new(0, 0), &Point2::new(size, size));
        let (base, pack) = pack_rects(&base, dims);
        let mut data = vec::from_elem(base.volume() as uint, 0u8);

        let w = base.dim().x;
        for i in range(0, glyphs.len()) {
            let g = &glyphs[i];
            let rect = pack[i];
            for y in range(0, g.height) {
                for x in range(0, g.width) {
                    data[(x + rect.min().x) + (y + rect.min().y) * w] = g.pixels[x + g.width * y]
                }
            }
        }

        // Scale into texture coordinates in [0.0, 1.0].
        let pack = pack.map(|int_rect| scale_rect(&base, int_rect));

        Fonter {
            lookup: ~range(0, dims.len()).map(
                        |i| (char::from_u32((start_char + i) as u32).unwrap(), pack[i])).collect(),
            texture: ~Texture::new_alpha(base.dim().x as uint, base.dim().y as uint, data),
        }
    }

    pub fn bind(&self) {
        self.texture.bind();
    }
}
