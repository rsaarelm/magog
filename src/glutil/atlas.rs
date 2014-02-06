use std::iter::AdditiveIterator;
use std::num::{min, max, sqrt, next_power_of_two};
use std::vec;
use std::iter::Iterator;

use cgmath::aabb::{Aabb, Aabb2};
use cgmath::point::{Point, Point2};
use cgmath::vector::Vec2;

use calx::pack_rect::pack_rects;
use calx::rectutil::RectUtil;
use texture::Texture;
use stb;

#[deriving(Clone)]
pub struct Sprite {
    priv bounds: Aabb2<int>,
    priv data: ~[u8],
}

impl Sprite {
    // Only supporting alpha channel for now.
    pub fn new_alpha(bounds: &Aabb2<int>, data: ~[u8]) -> Sprite {
        let bpp = 1;
        assert!(data.len() / bpp == bounds.volume() as uint);

        let mut ret = Sprite {
            bounds: *bounds,
            data: data
        };
        ret.crop();
        ret
    }

    #[inline]
    pub fn contains(&self, pos: &Point2<int>) -> bool {
        self.bounds.contains(pos)
    }

    #[inline]
    pub fn at(&self, pos: &Point2<int>) -> u8 {
        if self.contains(pos) {
            self.data[self.bounds.scan_pos(pos)]
        } else {
            0u8
        }
    }

    pub fn crop(&mut self) {
        let (mut min_x, mut min_y) = (self.bounds.max().x - 1, self.bounds.max().y - 1);
        let (mut max_x, mut max_y) = (self.bounds.min().x, self.bounds.min().y);
        for p in self.bounds.points() {
            if self.at(&p) > 0u8 {
                min_x = min(min_x, p.x);
                min_y = min(min_y, p.y);
                max_x = max(max_x, p.x);
                max_y = max(max_y, p.y);
            }
        }
        if min_x >= max_x || min_y >= max_y {
            // Empty area.
            self.data = ~[];
            self.bounds = Aabb2::new(&Point2::new(0, 0), &Point2::new(0, 0));
            return;
        }
        // TODO: Shrink bounds, create new data array.
    }
}

pub struct AtlasRect {
    bounds: Aabb2<f32>,
    texcoords: Aabb2<f32>,
}

impl AtlasRect {
    pub fn new(
        bounds_intrect: &Aabb2<int>,
        tex_intrect: &Aabb2<int>,
        tex_dim: &Vec2<int>) -> AtlasRect {
        let tex_scale = Vec2::new(1f32 / tex_dim.x as f32, 1f32 / tex_dim.y as f32);

        return AtlasRect {
            bounds: to_float_rect(bounds_intrect, &Vec2::new(1f32, 1f32)),
            texcoords: to_float_rect(tex_intrect, &tex_scale),
        };

        fn to_float_rect(rect: &Aabb2<int>, scale: &Vec2<f32>) -> Aabb2<f32> {
            Aabb2::new(
                &Point2::new(
                    rect.min().x as f32 * scale.x,
                    rect.min().y as f32 * scale.y),
                &Point2::new(
                    rect.max().x as f32 * scale.x,
                    rect.max().y as f32 * scale.y))
        }
    }
}

pub struct Atlas {
    sprites: ~[~Sprite],
    rects: ~[AtlasRect],
    is_dirty: bool,
    texture: ~Texture,
}

impl Atlas {
    pub fn new() -> Atlas {
        Atlas {
            sprites: ~[],
            rects: ~[],
            is_dirty: true,
            texture: ~Texture::new_alpha(0, 0, []),
        }
    }

    pub fn dirty(&mut self) {
        self.is_dirty = true;
    }

    pub fn clean(&mut self) {
        // Already good?
        if !self.is_dirty { return; }

        // Create gaps between the sprites to prevent edge artifacts.
        let dims = self.sprites.map(|s| s.bounds.dim() + Vec2::new(1, 1));
        let total_volume = dims.iter().map(|&v| v.x * v.y).sum();
        let atlas_dim = next_power_of_two(sqrt(total_volume as f64) as uint) as int;

        let base = Aabb2::new(&Point2::new(0, 0), &Point2::new(atlas_dim, atlas_dim));
        let (base, pack) = pack_rects(&base, dims);
        // Cut off the extra padding
        let pack : ~[Aabb2<int>] = pack.iter().map(|&rect| Aabb2::new(
                rect.min(), &rect.max().add_v(&Vec2::new(-1, -1)))).collect();

        let mut tex_data = vec::from_elem(base.volume() as uint, 0u8);

        assert!(pack.len() == self.sprites.len());
        self.rects = ~[];

        for i in range(0, self.sprites.len()) {
            paint_sprite(
                self.sprites[i], tex_data, &pack[i].min().to_vec(), base.dim().x);
            self.rects.push(AtlasRect::new(
                    &self.sprites[i].bounds, &pack[i], &base.dim()));
        }

        self.texture = ~Texture::new_alpha(base.dim().x as uint, base.dim().y as uint, tex_data);
        self.is_dirty = false;

        fn paint_sprite(sprite: &Sprite, tex_data: &mut [u8], offset: &Vec2<int>, tex_pitch: int) {
            let offset = offset - sprite.bounds.min().to_vec();
            for p in sprite.bounds.points() {
                tex_data[p.x + offset.x + (p.y + offset.y) * tex_pitch] = sprite.at(&p);
            }
        }
    }

    pub fn push(&mut self, sprite: ~Sprite) {
        self.dirty();
        self.sprites.push(sprite);
    }

    pub fn push_ttf(
        &mut self, ttf_data: ~[u8], size: f32,
        start_char: uint, num_chars: uint) {
        let font = stb::truetype::Font::new(ttf_data).expect("Bad ttf data.");
        for i in range(start_char, start_char + num_chars) {
            let glyph = font.glyph(i, size).expect("Font missing expected char");
            let min = Point2::new(glyph.xOffset as int, glyph.yOffset as int);
            let max = min.add_v(&Vec2::new(glyph.width, glyph.height));
            self.push(~Sprite::new_alpha(&Aabb2::new(&min, &max), glyph.pixels));
        }
    }

    pub fn len(&self) -> uint {
        self.sprites.len()
    }

    pub fn get(&mut self, i: uint) -> AtlasRect {
        self.clean();
        self.rects[i]
    }

    pub fn bind(&mut self) {
        self.clean();
        self.texture.bind();
    }
}
