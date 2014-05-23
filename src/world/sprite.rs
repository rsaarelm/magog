use cgmath::point::{Point2};
use color::rgb::{RGB, ToRGB};
use engine::{Engine, Image};

pub static FLOOR_Z: f32 = 0.500f32;
pub static BLOCK_Z: f32 = 0.400f32;

pub struct Sprite {
    pub image: Image,
    pub pos: Point2<f32>,
    pub z: f32,
    pub color: RGB<u8>,
}

impl Sprite {
    pub fn new(image: Image, pos: Point2<f32>, z: f32, color: RGB<u8>) -> Sprite {
        Sprite { image: image, pos: pos, z: z, color: color }
    }
}

impl Sprite {
    pub fn draw(&self, ctx: &mut Engine) {
        ctx.set_layer(self.z);
        ctx.set_color(&self.color);
        ctx.draw_image(&self.image, &self.pos);
    }
}

/// Interface for sprite-drawing.
pub trait DrawContext {
    fn draw<C: ToRGB>(
        &mut self, idx: uint, pos: &Point2<f32>, z: f32, color: &C);
}
