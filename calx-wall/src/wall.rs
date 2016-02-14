use std::u16;
use glium;
use calx_color::Rgba;
use calx_cache::{Atlas, AtlasBuilder, AtlasItem};
use calx_window::Displayable;

/// Collect textured 2D mesh elements that make up a single rendered frame.
pub struct Wall {
    shader: glium::Program,
    texture: glium::texture::Texture2d,
    meshes: Vec<Mesh>,
    /// Geometries of the atlas subitems.
    ///
    /// Stored heae next to the atlas texture for convenience, not actually
    /// used by the internal Wall logic.
    pub tiles: Vec<AtlasItem>,
}

impl Wall {
    pub fn new(display: &glium::Display, a: AtlasBuilder) -> Wall {
        let Atlas {
            image: img,
            items: tiles,
        } = a.build();

        let atlas_dim = img.dimensions();
        let tex_image =
            glium::texture::RawImage2d::from_raw_rgba(img.into_raw(),
                                                      atlas_dim);

        Wall {
            shader: glium::Program::from_source(display,
                                                include_str!("sprite.vert"),
                                                include_str!("sprite.frag"),
                                                None)
                        .unwrap(),
            texture: glium::texture::Texture2d::new(display, tex_image)
                         .unwrap(),
            meshes: vec![Mesh::new()],
            tiles: tiles,
        }
    }

    pub fn add_mesh(&mut self, vertices: Vec<Vertex>, faces: Vec<[u16; 3]>) {
        assert!(self.meshes.len() > 0);

        if self.meshes[self.meshes.len() - 1].is_full() {
            self.meshes.push(Mesh::new());
        }
        let idx = self.meshes.len() - 1;
        self.meshes[idx].push(vertices, faces);
    }
}

impl Displayable for Wall {
    fn display<S>(&mut self, display: &glium::Display, target: &mut S)
        where S: glium::Surface
    {
        let (w, h) = target.get_dimensions();
        let uniforms = glium::uniforms::UniformsStorage::new("tex",
            glium::uniforms::Sampler(&self.texture, glium::uniforms::SamplerBehavior {
                magnify_filter: glium::uniforms::MagnifySamplerFilter::Nearest,
                .. Default::default() }))
            .add("canvas_size", [w as f32, h as f32]);

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
            target.draw(&vertices, &indices, &self.shader, &uniforms, &params)
                  .unwrap();
        }
        self.meshes = vec![Mesh::new()];
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
        // limit it's time to start a new mesh.
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
    pub fn new<V, W>(pos: V,
                     z: f32,
                     tex_coord: W,
                     color: Rgba,
                     back_color: Rgba)
                     -> Vertex
        where V: Into<[f32; 2]>,
              W: Into<[f32; 2]>
    {
        let pos = pos.into();
        let tex_coord = tex_coord.into();
        Vertex {
            pos: [pos[0], pos[1], z],
            tex_coord: tex_coord,
            color: [color.r, color.g, color.b, color.a],
            back_color: [back_color.r,
                         back_color.g,
                         back_color.b,
                         back_color.a],
        }
    }
}
