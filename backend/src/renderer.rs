use std::default::Default;
use std::mem;
use glium;
use glium::texture;
use util::{Color, Rect, V2};

pub struct Renderer {
    shader: glium::Program,
    texture: texture::Texture2d,
    params: glium::DrawParameters,

    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl Renderer {
    pub fn new<T>(display: &glium::Display, texture_image: T) -> Renderer
        where T: texture::Texture2dData {
        let shader = glium::Program::from_source(display,
            // Vertex
            "
                #version 120

                attribute vec3 pos;
                attribute vec4 color;
                attribute vec2 tex_coord;

                varying vec2 v_tex_coord;
                varying vec4 v_color;

                void main() {
                    v_tex_coord = tex_coord;
                    v_color = color;
                    gl_Position = vec4(pos, 1.0);
                }
            ",
            // Fragment
            "
                #version 120

                uniform sampler2D texture;

                varying vec2 v_tex_coord;
                varying vec4 v_color;

                void main() {
                    vec4 tex_color = texture2D(texture, v_tex_coord);
                    gl_FragColor = v_color * tex_color;
                }
            ",
            None).unwrap();
        let texture = texture::Texture2d::new(display, texture_image);

        let mut params: glium::DrawParameters = Default::default();
        params.backface_culling = glium::BackfaceCullingMode::CullCounterClockWise;
        params.depth_function = glium::DepthFunction::IfLessOrEqual;
        params.blending_function = Some(glium::BlendingFunction::LerpBySourceAlpha);

        Renderer {
            shader: shader,
            texture: texture,
            params: params,
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn clear<C: Color>(&mut self, color: &C) {
        self.vertices = Vec::new();
        self.indices = Vec::new();
        // TODO
    }

    pub fn scissor(&mut self, Rect(V2(ax, ay), V2(aw, ah)): Rect<u32>) {
        // TODO
    }

    pub fn set_window_size(&mut self, (w, h): (i32, i32)) {
        // TODO
    }

    /// Draw the accumulated geometry and clear the accum buffers.
    pub fn draw<S>(&mut self, display: &glium::Display, target: &mut S)
        where S: glium::Surface {
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target.clear_depth(1.0);
        // Extract the geometry accumulation buffers and convert into
        // temporary Glium buffers.
        let vertices = glium::VertexBuffer::new(
            display, mem::replace(&mut self.vertices, Vec::new()));
        let indices = glium::IndexBuffer::new(
            display,
            glium::index_buffer::TrianglesList(
                mem::replace(&mut self.indices, Vec::new())));

        let uniforms = glium::uniforms::UniformsStorage::new("texture",
            glium::uniforms::Sampler(&self.texture, glium::uniforms::SamplerBehavior {
                magnify_filter: glium::uniforms::MagnifySamplerFilter::Nearest,
                .. Default::default() }));

        target.draw(&vertices, &indices, &self.shader, &uniforms, &self.params);
    }
}

#[vertex_format]
#[derive(Copy)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub color: [f32; 4],
    pub tex_coord: [f32; 2],
}
