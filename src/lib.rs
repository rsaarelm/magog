extern crate euclid;

use std::mem;

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
        Context {
            draw_list: Vec::new(),
        }
    }

    pub fn begin_frame(&mut self) {
        unimplemented!();
    }

    pub fn text(&mut self, text: &str) {
        unimplemented!();
    }

    pub fn button(&mut self, caption: &str) -> bool {
        unimplemented!();
    }

    pub fn end_frame(&mut self) -> Vec<DrawBatch<T>> {
        let mut ret = Vec::new();
        mem::swap(&mut ret, &mut self.draw_list);
        ret
    }
}

/// A sequence of primitive draw operarations.
pub struct DrawBatch<T> {
    /// Texture used for the current batch, details depend on backend
    /// implementation
    pub texture_id: T,
    /// Clipping rectangle for the current batch
    pub clip: euclid::Rect<f32>,
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
