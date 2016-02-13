use std::rc::Rc;
use std::default::Default;
use std::u16;
use glium;
use image;
use calx_color::Rgba;
use RenderTarget;

/// Collect textured mesh elements that make up a single rendered frame.
pub struct Buffer {
    shader: Rc<glium::Program>,
    texture: Rc<glium::texture::Texture2d>,
    meshes: Vec<Mesh>,
}

impl Buffer {
    /// Create a mesh render buffer with shared shader and texture.
    pub fn new_shared(shader: Rc<glium::Program>,
                      texture: Rc<glium::texture::Texture2d>)
                      -> Buffer {
        Buffer {
            shader: shader,
            texture: texture,
            meshes: vec![Mesh::new()],
        }
    }

    /// Create a mesh render buffer with locally owned shader and texture.
    /// This uses the default sprite shader.
    pub fn new<'a, P>(display: &glium::Display,
                      atlas_image: image::ImageBuffer<P, Vec<u8>>)
                      -> Buffer
        where P: image::Pixel<Subpixel = u8> + 'static
    {
        // where T: glium::texture::Texture2dDataSource<'a> {
        let atlas_dim = atlas_image.dimensions();
        let tex_image =
            glium::texture::RawImage2d::from_raw_rgba(atlas_image.into_raw(),
                                                      atlas_dim);

        Buffer::new_shared(
            Rc::new(glium::Program::from_source(
                display,
                include_str!("sprite.vert"),
                include_str!("sprite.frag"),
                None).unwrap()),
            Rc::new(glium::texture::Texture2d::new(
                display, tex_image).unwrap()))
    }

    pub fn flush<S>(&mut self, display: &glium::Display, target: &mut S)
        where S: glium::Surface
    {
        let uniforms = glium::uniforms::UniformsStorage::new("tex",
            glium::uniforms::Sampler(&*self.texture, glium::uniforms::SamplerBehavior {
                magnify_filter: glium::uniforms::MagnifySamplerFilter::Nearest,
                .. Default::default() }));

        let params = glium::DrawParameters {
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullCounterClockwise,
            depth: glium::Depth {
                test: glium::DepthTest::IfLessOrEqual,
                write: true,
                .. Default::default()
            },
            blend: glium::Blend::alpha_blending(),
            .. Default::default() };

        // Extract the geometry accumulation buffers and convert into
        // temporary Glium buffers.
        for mesh in self.meshes.iter() {
            let vertices = glium::VertexBuffer::new(display, &mesh.vertices)
                               .unwrap();
            let indices = glium::IndexBuffer::new(
                display, glium::index::PrimitiveType::TrianglesList, &mesh.indices).unwrap();
            target.draw(&vertices, &indices, &*self.shader, &uniforms, &params)
                  .unwrap();
        }
        self.meshes = vec![Mesh::new()];
    }
}

impl RenderTarget for Buffer {
    fn add_mesh(&mut self, vertices: Vec<Vertex>, faces: Vec<[u16; 3]>) {
        assert!(self.meshes.len() > 0);

        if self.meshes[self.meshes.len() - 1].is_full() {
            self.meshes.push(Mesh::new());
        }
        let idx = self.meshes.len() - 1;
        self.meshes[idx].push(vertices, faces);
    }
}

/// On-screen geometry data.
struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl Mesh {
    pub fn new() -> Mesh {
        Mesh {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn is_full(&self) -> bool {
        // When you're getting within an order of magnitude of the u16 max
        // limit it's time to flush.
        self.vertices.len() > 1 << 15
    }

    pub fn push(&mut self, vertices: Vec<Vertex>, faces: Vec<[u16; 3]>) {
        assert!(self.vertices.len() + vertices.len() < u16::MAX as usize);
        let offset = self.vertices.len() as u16;
        for v in vertices.into_iter() {
            self.vertices.push(v);
        }

        for face in faces.into_iter() {
            for i in face.into_iter() {
                self.indices.push(i + offset);
            }
        }
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

impl Vertex {
    #[inline(always)]
    pub fn new<V>(pos: V,
                  z: f32,
                  tex_coord: V,
                  color: Rgba,
                  back_color: Rgba)
                  -> Vertex
        where V: Into<[f32; 2]>
    {
        let pos = pos.into();
        let tex_coord = tex_coord.into();
        Vertex {
            pos: [pos[0], pos[1], z],
            tex_coord: tex_coord,
            color: [color.r, color.g, color.b, color.a],
            back_color: [back_color.r, back_color.g, back_color.b, back_color.a],
        }
    }
}
