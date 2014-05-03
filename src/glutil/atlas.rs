use std::vec::Vec;
use std::iter::AdditiveIterator;
use std::num::{next_power_of_two};
use std::cmp::{min, max};
use std::iter::Iterator;

use cgmath::aabb::{Aabb, Aabb2};
use cgmath::point::{Point, Point2};
// cgmath Vector shadows std::vec::Vector and breaks as_slice...
use cgVector = cgmath::vector::Vector;
use cgmath::vector::Vector2;
use hgl::texture::{Texture, ImageInfo};
use hgl::texture;
use hgl::texture::pixel;
use gl::types::{GLint};

use pack_rect::pack_rects;
use rectutil::RectUtil;
use tile::{Tile, TILE_ALPHA};
use stb;

pub struct AtlasRect {
    pub bounds: Aabb2<f32>,
    pub texcoords: Aabb2<f32>,
}

impl AtlasRect {
    pub fn new(
        bounds_intrect: &Aabb2<int>,
        tex_intrect: &Aabb2<int>,
        tex_dim: &Vector2<int>) -> AtlasRect {
        let tex_scale = Vector2::new(1f32 / tex_dim.x as f32, 1f32 / tex_dim.y as f32);
        return AtlasRect {
            bounds: to_float_rect(bounds_intrect, &Vector2::new(1f32, 1f32)),
            texcoords: to_float_rect(tex_intrect, &tex_scale),
        };

        fn to_float_rect(rect: &Aabb2<int>, scale: &Vector2<f32>) -> Aabb2<f32> {
            RectUtil::new(
                rect.min().x as f32 * scale.x, rect.min().y as f32 * scale.y,
                rect.max().x as f32 * scale.x, rect.max().y as f32 * scale.y)
        }
    }
}

pub struct Atlas {
    tiles: Vec<Tile>,
    rects: Vec<AtlasRect>,
    is_dirty: bool,
    texture: Texture,
}

impl Atlas {
    pub fn new() -> Atlas {
        let tex = Texture::new_raw(texture::Texture2D);
        tex.filter(texture::Nearest);
        tex.wrap(texture::ClampToEdge);
        Atlas {
            tiles: vec!(),
            rects: vec!(),
            is_dirty: true,
            texture: tex,
        }
    }

    pub fn dirty(&mut self) {
        self.is_dirty = true;
    }

    pub fn clean(&mut self) {
        // Already good?
        if !self.is_dirty { return; }

        // Create gaps between the tiles to prevent edge artifacts.
        let dims = self.tiles.iter().map(|s| s.bounds.dim() + Vector2::new(1, 1)).collect::<Vec<Vector2<int>>>();
        let total_volume = dims.iter().map(|&v| v.x * v.y).sum();
        let atlas_dim = next_power_of_two((total_volume as f64).sqrt() as uint) as int;

        let base = RectUtil::new(0, 0, atlas_dim, atlas_dim);
        let (base, pack) = pack_rects(&base, dims.as_slice());
        // Cut off the extra padding
        let pack : Vec<Aabb2<int>> = pack.iter().map(|&rect| Aabb2::new(
                *rect.min(), rect.max().add_v(&Vector2::new(-1, -1)))).collect();

        let mut tex_data = Vec::from_elem(base.volume() as uint, 0u8);

        assert!(pack.len() == self.tiles.len());
        self.rects = vec!();

        for i in range(0, self.tiles.len()) {
            paint_tile(
                self.tiles.get(i), &mut tex_data, &pack.get(i).min().to_vec(), base.dim().x);
            self.rects.push(AtlasRect::new(
                    &self.tiles.get(i).bounds, pack.get(i), &base.dim()));
        }

        let info = ImageInfo::new()
            .width(base.dim().x as GLint)
            .height(base.dim().y as GLint)
            .pixel_format(pixel::RED)
            .pixel_type(pixel::UNSIGNED_BYTE)
            ;
        self.texture.load_image(info, tex_data.get(0));

        self.is_dirty = false;

        fn paint_tile(
            tile: &Tile, tex_data: &mut Vec<u8>,
            offset: &Vector2<int>, tex_pitch: int) {
            let offset = offset - tile.bounds.min().to_vec();
            for p in tile.bounds.points() {
                tex_data.grow_set(
                    (p.x + offset.x + (p.y + offset.y) * tex_pitch) as uint,
                    &0, tile.at(&p));
            }
        }
    }

    pub fn push(&mut self, tile: Tile) -> uint {
        self.dirty();
        self.tiles.push(tile);
        self.tiles.len() - 1
    }

    pub fn push_ttf(
        &mut self, ttf_data: Vec<u8>, size: f32,
        start_char: uint, num_chars: uint) {
        let font = stb::truetype::Font::new(ttf_data).expect("Bad ttf data.");
        for i in range(start_char, start_char + num_chars) {
            let mut glyph = font.glyph(i, size).expect("Font missing expected char");

            // Convert black alpha from STB to our TILE_ALPHA.
            for i in range(0, glyph.pixels.len()) {
                if *glyph.pixels.get(i) == 0 {
                    glyph.pixels.grow_set(i, &0, TILE_ALPHA);
                }
            }

            let min = Point2::new(glyph.xOffset as int, glyph.yOffset as int);
            let max = min.add_v(&Vector2::new(glyph.width, glyph.height));
            self.push(Tile::new_alpha(Aabb2::new(min, max), glyph.pixels));
        }
    }

    pub fn len(&self) -> uint {
        self.tiles.len()
    }

    pub fn get(&mut self, i: uint) -> AtlasRect {
        self.clean();
        *self.rects.get(i)
    }

    pub fn bind(&mut self) {
        self.clean();
        self.texture.bind();
    }
}
