extern crate euclid;

use std::mem;
use euclid::{Rect, Point2D, Size2D};

/// An immediate mode graphical user interface context.
///
/// The context persists over a frame and receives commands that combine GUI
/// description and input handling. At the end of the frame, the commands are
/// converted into rendering instructions for the GUI.
pub struct Context<T> {
    draw_list: Vec<DrawBatch<T>>,
}

impl<T> Context<T> {
    pub fn new() -> Context<T> {
        Context { draw_list: Vec::new() }
    }

    pub fn begin_frame(&mut self) {
        // TODO
    }

    pub fn text(&mut self, text: &str) {
        unimplemented!();
    }

    pub fn button(&mut self, caption: &str) -> bool {
        unimplemented!();
    }

    pub fn demo(&mut self, tex: T) {
        // TODO: Temporary crap, remove in favor of actual stuff.
        let vertices = vec![
            Vertex { pos: [ 10.0, 10.0], color: [0.0, 1.0, 0.0, 1.0], tex: [0.0, 0.0] },
            Vertex { pos: [ 500.0, 0.0], color: [0.0, 0.0, 1.0, 1.0], tex: [0.0, 0.0] },
            Vertex { pos: [ 0.0, 500.0], color: [1.0, 0.0, 0.0, 1.0], tex: [0.0, 0.0] },
        ];

        self.draw_list.push(DrawBatch {
            texture_id: tex,
            clip: None,
            vertices: vertices,
            triangle_indices: vec![0, 1, 2],
        });
    }

    pub fn end_frame(&mut self) -> Vec<DrawBatch<T>> {
        let mut ret = Vec::new();
        mem::swap(&mut ret, &mut self.draw_list);
        ret
    }

    /// Register mouse button state.
    pub fn input_mouse_button(&mut self, id: ButtonId, x: i32, y: i32, is_down: bool) {
        // TODO
    }

    /// Register mouse motion.
    pub fn input_mouse_move(&mut self, x: i32, y: i32) {
        // TODO
    }

    /// Register printable character input.
    pub fn input_char(&mut self, c: char) {
        // TODO
    }

    /// Register a nonprintable key state.
    pub fn input_key_state(&mut self, k: Keycode, is_down: bool) {
        // TODO
    }

    /// Build a font atlas from a TTF and construct a texture object.
    ///
    /// TODO: Font customization, point size, character ranges.
    pub fn init_font<F>(&mut self, ttf_data: &[u8], register_texture: F) -> Result<Font, ()>
        where F: FnOnce(&[u8], u32, u32) -> T
    {
        unimplemented!();
    }
}

/// A sequence of primitive draw operarations.
pub struct DrawBatch<T> {
    /// Texture used for the current batch, details depend on backend
    /// implementation
    pub texture_id: T,
    /// Clipping rectangle for the current batch
    pub clip: Option<Rect<f32>>,
    /// Vertex data
    pub vertices: Vec<Vertex>,
    /// Indices into the vertex array for the triangles that make up the batch
    pub triangle_indices: Vec<u16>,
}

#[derive(Copy, Clone)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub color: [f32; 4],
    pub tex: [f32; 2],
}

/// Text alignment.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Align {
    Left,
    Center,
    Right,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ButtonId {
    Left,
    Middle,
    Right,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Keycode {
    Tab,
    Shift,
    Ctrl,
    Enter,
    Backspace,
    Del,
    Up,
    Down,
    Left,
    Right,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Font(u64);
