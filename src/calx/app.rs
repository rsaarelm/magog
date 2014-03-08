use std::default::Default;
use color::RGB;
use color::rgb::{ToRGB};
use cgmath::aabb::{Aabb2};
use cgmath::vector::{Vec2};
use cgmath::point::{Point, Point2};
use rectutil::RectUtil;
use color::rgb::consts::*;
use renderer;
use renderer::{Renderer};
use gen_id::CodeId;

static FONT_WIDTH: f32 = 8.0;
static FONT_START_CHAR: uint = 32;
static FONT_NUM_CHARS: uint = 96;
pub static FONT_HEIGHT: f32 = 8.0;
pub static FONT_SPACE: f32 = FONT_WIDTH;

pub struct App<R> {
    r: ~R,
    draw_color: RGB<u8>,
    hot_item: CodeId,
    active_item: CodeId,
    alive: bool,
    resolution: Vec2<f32>,
}

impl <R: Renderer> App<R> {
    pub fn new(width: uint, height: uint, title: &str) -> App<R> {
        App {
            r: ~Renderer::new(width, height, title),
            draw_color: WHITE,
            hot_item: Default::default(),
            active_item: Default::default(),
            alive: true,
            resolution: Vec2::new(width as f32, height as f32),
        }
    }

    pub fn string_bounds(&mut self, text: &str) -> Aabb2<f32> {
        RectUtil::new(0f32, 0f32, text.len() as f32 * FONT_WIDTH, -FONT_HEIGHT)
    }

    pub fn draw_string<C: ToRGB>(&mut self, pos: &Point2<f32>, z: f32, color: &C, text: &str) {
        let first_font_idx : uint = 1;

        let mut pos = *pos;
        for c in text.chars() {
            let i = c as u32;
            if i >= FONT_START_CHAR as u32
                && i < (FONT_START_CHAR + FONT_NUM_CHARS) as u32 {
                self.r.draw_sprite(i as uint - FONT_START_CHAR + first_font_idx, &pos, z, color, renderer::AlphaDraw);
            }
            pos = pos.add_v(&Vec2::new(FONT_WIDTH, 0.0));
        }
    }

    pub fn screen_area(&self) -> Aabb2<f32> {
        RectUtil::new(0f32, 0f32, self.resolution.x, self.resolution.y)
    }
}
