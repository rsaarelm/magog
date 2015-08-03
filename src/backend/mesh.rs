use std::rc::{Rc};
use std::default::{Default};
use std::u16;
use glium;

pub struct Buffer {
    shader: Rc<glium::Program>,
    texture: Rc<glium::texture::Texture2d>,
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl Buffer {
    pub fn new(shader: Rc<glium::Program>, texture: Rc<glium::texture::Texture2d>) -> Buffer {
        Buffer {
            shader: shader,
            texture: texture,
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn should_flush(&self) -> bool {
        // When you're getting within an order of magnitude of the u16 max
        // limit it's time to flush.
        self.vertices.len() > 1<<15
    }

    pub fn push(&mut self, vertices: Vec<Vertex>, faces: Vec<[u16; 3]>) {
        assert!(self.vertices.len() + vertices.len() < u16::MAX as usize);
        let offset = self.vertices.len() as u16;
        for v in vertices.into_iter() { self.vertices.push(v); }

        for face in faces.into_iter() {
            for i in face.into_iter() {
                self.indices.push(i + offset);
            }
        }
    }

    pub fn flush<S>(&mut self, display: &glium::Display, target: &mut S)
        where S: glium::Surface {
        // Extract the geometry accumulation buffers and convert into
        // temporary Glium buffers.
        let vertices = glium::VertexBuffer::new(display, &self.vertices).unwrap();
        let indices = glium::IndexBuffer::new(
            display, glium::index::PrimitiveType::TrianglesList, &self.indices).unwrap();

        let uniforms = glium::uniforms::UniformsStorage::new("tex",
            glium::uniforms::Sampler(&*self.texture, glium::uniforms::SamplerBehavior {
                magnify_filter: glium::uniforms::MagnifySamplerFilter::Nearest,
                .. Default::default() }));

        let params = glium::DrawParameters {
            backface_culling: glium::BackfaceCullingMode::CullCounterClockWise,
            depth_test: glium::DepthTest::IfLessOrEqual,
            depth_write: true,
            blending_function: Some(glium::BlendingFunction::Addition {
                source: glium::LinearBlendingFactor::SourceAlpha,
                destination: glium::LinearBlendingFactor::OneMinusSourceAlpha }),
            .. Default::default() };

        target.draw(&vertices, &indices, &*self.shader, &uniforms, &params).unwrap();

        self.vertices.clear();
        self.indices.clear();
    }
}

#[derive(Copy, Clone)]
/// Geometry vertex in on-screen graphics.
pub struct Vertex {
    /// Coordinates on screen
    pub pos: [f32; 3],
    /// Texture coordinates
    pub tex_coord: [f32; 2],
    /// Color for the light parts of the texture
    pub color: [f32; 4],
    /// Color for the dark parts of the texture
    pub back_color: [f32; 4],
}
implement_vertex!(Vertex, pos, tex_coord, color, back_color);
