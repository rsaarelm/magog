use cgmath::point::{Point2};
use color::rgb::RGB;
use calx::app;
use calx::app::App;
use calx::renderer::Renderer;
use calx::renderer;

pub static FLOOR_Z: f32 = 0.500f32;
pub static BLOCK_Z: f32 = 0.400f32;

pub struct Sprite {
    pub idx: uint,
    pub pos: Point2<f32>,
    pub z: f32,
    pub color: RGB<u8>,
}

impl Sprite {
    pub fn new(idx: uint, pos: Point2<f32>, z: f32, color: RGB<u8>) -> Sprite {
        Sprite { idx: idx, pos: pos, z: z, color: color }
    }
}

impl<R: Renderer> Sprite {
    pub fn draw(&self, app: &mut App<R>) {
        app.r.draw_tile(self.idx, &self.pos, self.z, &self.color, renderer::ColorKeyDraw);
    }
}

pub fn tile(idx: uint) -> uint { idx + app::SPRITE_INDEX_START }
