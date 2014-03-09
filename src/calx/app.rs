use std::cmp::max;
use std::default::Default;
use color::RGB;
use color::rgb::{ToRGB};
use cgmath::aabb::{Aabb, Aabb2};
use cgmath::vector::{Vec2};
use cgmath::point::{Point, Point2};
use stb::image::Image;
use rectutil::RectUtil;
use color::rgb::consts::*;
use renderer;
use renderer::{Renderer};
use gen_id::CodeId;
use sprite::Sprite;

static FONT_WIDTH: f32 = 8.0;
static FONT_START_CHAR: uint = 32;
static FONT_NUM_CHARS: uint = 96;
pub static FONT_HEIGHT: f32 = 8.0;
pub static FONT_SPACE: f32 = FONT_WIDTH;

// XXX: Hardcoded assumption in a couple places that the renderer
// will use sprite 1 for its own purposes.
pub static SPRITE_INDEX_START: uint = FONT_NUM_CHARS + 1;

pub enum Align {
    Left,
    Center,
    Right
}

pub struct App<R> {
    r: ~R,
    draw_color: RGB<u8>,
    background_color: RGB<u8>,
    draw_layer: f32,
    hot_item: CodeId,
    active_item: CodeId,
    alive: bool,
    resolution: Vec2<f32>,
}

impl <R: Renderer> App<R> {
    pub fn new(width: uint, height: uint, title: &str) -> App<R> {
        let mut ret : App<R> = App {
            r: ~Renderer::new(width, height, title),
            draw_color: WHITE,
            background_color: BLACK,
            draw_layer: 0f32,
            hot_item: Default::default(),
            active_item: Default::default(),
            alive: true,
            resolution: Vec2::new(width as f32, height as f32),
        };

        // Load font.
        let font = Image::load("assets/font.png", 1).unwrap();
        let sprites = Sprite::new_alpha_set(
            &Vec2::new(FONT_WIDTH as int, FONT_HEIGHT as int),
            &Vec2::new(font.width as int, font.height as int),
            font.pixels,
            &Vec2::new(0, -FONT_HEIGHT as int));
        for i in range(0, FONT_NUM_CHARS) {
            ret.r.add_sprite(~sprites[i].clone());
        }

        ret
    }

    pub fn string_bounds(&mut self, text: &str) -> Aabb2<f32> {
        RectUtil::new(0f32, 0f32, text.len() as f32 * FONT_WIDTH, -FONT_HEIGHT)
    }

    pub fn set_color<C: ToRGB>(&mut self, color: &C) {
        self.draw_color = color.to_rgb();
    }

    pub fn set_background<C: ToRGB>(&mut self, color: &C) {
        self.background_color = color.to_rgb();
    }

    pub fn set_layer(&mut self, z: f32) {
        self.draw_layer = z;
    }

    pub fn draw_sprite(&mut self, idx: uint, pos: &Point2<f32>) {
        self.r.draw_sprite(idx, pos, self.draw_layer, &self.draw_color, renderer::ColorKeyDraw);
    }

    pub fn _draw_string<C: ToRGB>(&mut self, pos: &Point2<f32>, color: &C, text: &str) {
        let first_font_idx : uint = 1;

        let mut pos = *pos;
        for c in text.chars() {
            let i = c as u32;
            if i >= FONT_START_CHAR as u32
                && i < (FONT_START_CHAR + FONT_NUM_CHARS) as u32 {
                self.r.draw_sprite(i as uint - FONT_START_CHAR + first_font_idx, &pos,
                    self.draw_layer, color, renderer::ColorKeyDraw);
            }
            pos = pos.add_v(&Vec2::new(FONT_WIDTH, 0.0));
        }
    }

    pub fn print_words(&mut self, area: &Aabb2<f32>, align: Align, text: &str) {
        let words: ~[&str] = text.split(' ').collect();
        let bounds = words.map(|&w| self.string_bounds(w).dim().x as uint);
        let mut i = 0;
        let origin = area.min().add_v(&Vec2::new(0.0, FONT_HEIGHT));
        let width = area.dim().x;
        let max_lines = (area.dim().y / FONT_HEIGHT) as uint;
        let mut pos = origin;
        let mut line = 0;
        while i < words.len() && line < max_lines {
            let (n, len) = num_fitting_words(width as uint, FONT_SPACE as uint, bounds.slice(i, bounds.len()));
            let n = max(1, n);

            let diff = area.dim().x - len as f32;
            match align {
                Left => (),
                Center => { pos.x += diff / 2.0; },
                Right => { pos.x += diff; },
            }
            for j in range(i, i + n) {
                // XXX: Always using outline print.
                self.outline_string(&pos, words[j]);
                pos.x += bounds[j] as f32 + FONT_SPACE;
            }
            i += n;
            pos.x = origin.x;
            pos.y += FONT_HEIGHT;
            line += 1;
        }

        fn num_fitting_words(span: uint, space: uint, lengths: &[uint]) -> (uint, uint) {
            if lengths.len() == 0 { return (0, 0) }
            let mut total = lengths[0];
            for i in range(1, lengths.len()) {
                let new_total = total + space + lengths[i];
                if new_total > span {
                    return (i, total);
                }
                total = new_total;
            }
            return (lengths.len(), total);
        }
    }


    pub fn draw_string(&mut self, pos: &Point2<f32>, text: &str) {
        let fore = self.draw_color;
        self._draw_string(pos, &fore, text);
    }

    pub fn outline_string(&mut self, pos: &Point2<f32>, text: &str) {
        let back = self.background_color;
        self._draw_string(&pos.add_v(&Vec2::new( 1.0f32,  0.0f32)), &back, text);
        self._draw_string(&pos.add_v(&Vec2::new(-1.0f32,  0.0f32)), &back, text);
        self._draw_string(&pos.add_v(&Vec2::new( 0.0f32,  1.0f32)), &back, text);
        self._draw_string(&pos.add_v(&Vec2::new( 0.0f32, -1.0f32)), &back, text);
        self.draw_string(pos, text);
    }

    pub fn screen_area(&self) -> Aabb2<f32> {
        RectUtil::new(0f32, 0f32, self.resolution.x, self.resolution.y)
    }

    pub fn flush(&mut self) {
        self.r.flush();
        if !self.r.is_alive() {
            self.alive = false;
        }
    }
}
