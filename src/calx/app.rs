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
use tile::Tile;
use text;

static FONT_DATA: &'static [u8] = include!("../../gen/font_data.inc");
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
        let font = Image::load_from_memory(FONT_DATA, 1).unwrap();
        let tiles = Tile::new_alpha_set(
            &Vec2::new(FONT_WIDTH as int, FONT_HEIGHT as int),
            &Vec2::new(font.width as int, font.height as int),
            font.pixels,
            &Vec2::new(0, -FONT_HEIGHT as int));
        for i in range(0, FONT_NUM_CHARS) {
            ret.r.add_tile(~tiles.get(i).clone());
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

    pub fn draw_tile(&mut self, idx: uint, pos: &Point2<f32>) {
        self.r.draw_tile(idx, pos, self.draw_layer, &self.draw_color, renderer::ColorKeyDraw);
    }

    pub fn _draw_string<C: ToRGB>(&mut self, pos: &Point2<f32>, color: &C, text: &str) {
        let first_font_idx : uint = 1;

        let mut pos = *pos;
        for c in text.chars() {
            let i = c as u32;
            if i >= FONT_START_CHAR as u32
                && i < (FONT_START_CHAR + FONT_NUM_CHARS) as u32 {
                self.r.draw_tile(i as uint - FONT_START_CHAR + first_font_idx, &pos,
                    self.draw_layer, color, renderer::ColorKeyDraw);
            }
            pos = pos.add_v(&Vec2::new(FONT_WIDTH, 0.0));
        }
    }

    pub fn print_words(&mut self, area: &Aabb2<f32>, align: Align, text: &str) {
        let w = area.dim().x;
        if w < FONT_WIDTH { return; }
        let origin = area.min().add_v(&Vec2::new(0.0, FONT_HEIGHT));

        let mut pos = origin;
        let wrapped = text::wrap_lines((w / FONT_WIDTH) as uint, text);
        for line in wrapped.split('\n') {
            pos.x = origin.x;
            let diff = w - line.len() as f32 * FONT_WIDTH;
            let halfdiff = w - (line.len() / 2) as f32 * FONT_WIDTH;
            pos.x = match align {
                Left => origin.x,
                Center => origin.x + halfdiff,
                Right => origin.x + diff,
            };
            // XXX: Always using outline print.
            self.outline_string(&pos, line);
            pos.y += FONT_HEIGHT;
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
