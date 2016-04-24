extern crate euclid;

use std::mem;

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

pub struct DrawBatch<T> {
    pub texture_id: T,
    pub clip: euclid::Rect<f32>,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

#[derive(Copy, Clone)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub color: [f32; 4],
    pub tex: [f32; 2],
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Align {
    Left,
    Center,
    Right,
}
