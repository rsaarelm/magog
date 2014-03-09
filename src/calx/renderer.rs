use color::rgb::{ToRGB};
use cgmath::aabb::Aabb2;
use cgmath::point::Point2;
use sprite::Sprite;

#[deriving(Clone, Eq)]
pub struct KeyEvent {
    // Scancode (ignores local layout)
    code: uint,
    // Printable character (if any)
    ch: Option<char>,
}

#[deriving(Eq, Clone)]
pub struct MouseState {
    pos: Point2<f32>,
    left: bool,
    middle: bool,
    right: bool,
}

pub enum DrawMode {
    /// Draw opaque, but treat gray value 0x80 as totally transparent.
    ColorKeyDraw,
    /// Draw with alpha blending.
    AlphaDraw,
}

pub trait Renderer {
    fn new(width: uint, height: uint, title: &str) -> Self;
    fn fill_rect<C: ToRGB>(&mut self, rect: &Aabb2<f32>, z: f32, color: &C);
    fn add_sprite(&mut self, sprite: ~Sprite) -> uint;
    fn draw_sprite<C: ToRGB>(&mut self, idx: uint, pos: &Point2<f32>, z: f32, color: &C, mode: DrawMode);
    fn flush(&mut self);
    fn screenshot(&mut self, path: &str);
    fn pop_key(&mut self) -> Option<KeyEvent>;
    fn get_mouse(&self) -> MouseState;
    fn is_alive(&self) -> bool;
}
