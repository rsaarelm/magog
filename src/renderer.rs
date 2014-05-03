use color::rgb::{ToRGB};
use cgmath::aabb::Aabb2;
use cgmath::point::Point2;
use tile::Tile;

#[deriving(Clone, Eq)]
pub struct KeyEvent {
    // Scancode (ignores local layout)
    pub code: uint,
    // Printable character (if any)
    pub ch: Option<char>,
}

#[deriving(Eq, Clone)]
pub struct MouseState {
    pub pos: Point2<f32>,
    pub left: bool,
    pub middle: bool,
    pub right: bool,
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
    fn add_tile(&mut self, tile: Tile) -> uint;
    fn draw_tile<C: ToRGB>(&mut self, idx: uint, pos: &Point2<f32>, z: f32, color: &C, mode: DrawMode);
    fn flush(&mut self);
    fn screenshot(&mut self, path: &str);
    fn pop_key(&mut self) -> Option<KeyEvent>;
    fn get_mouse(&self) -> MouseState;
    fn is_alive(&self) -> bool;
}
