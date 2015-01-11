use glium::texture;
use util::{Color, Rect, V2};

pub struct Renderer;

impl Renderer {
    pub fn new() -> Renderer {
        Renderer
    }

    pub fn clear<C: Color>(&mut self, color: &C) {
        // TODO
    }

    pub fn scissor(&mut self, Rect(V2(ax, ay), V2(aw, ah)): Rect<u32>) {
        // TODO
    }

    pub fn set_window_size(&mut self, (w, h): (i32, i32)) {
        // TODO
    }
}

#[vertex_format]
#[derive(Copy)]
pub struct Vertex {
    #[name = "a_pos"]
    pub pos: [f32; 3],

    #[name = "a_color"]
    pub color: [f32; 4],

    #[name = "a_tex_coord"]
    pub tex_coord: [f32; 2],
}

#[uniforms]
struct Uniforms<'a> {
    s_texture: &'a texture::Texture2d,
}
