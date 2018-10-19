use cache;
use calx;
use euclid::Point2D;
use glium;
use std::error::Error;
use std::io;
pub use vitral::backend::KeyEvent;
use vitral::{self, backend, Color};

pub type Core = vitral::Core<Vertex>;

pub struct Backend {
    inner: backend::Backend<Vertex>,
}

impl Backend {
    pub fn start<S: Into<String>>(
        width: u32,
        height: u32,
        title: S,
    ) -> Result<Backend, Box<Error>> {
        const SHADER: glium::program::SourceCode = glium::program::SourceCode {
            vertex_shader: "
            #version 150 core

            uniform mat4 matrix;

            in vec2 pos;
            in vec4 color;
            in vec4 back_color;
            in vec2 tex_coord;

            out vec4 v_color;
            out vec4 v_back_color;
            out vec2 v_tex_coord;

            void main() {
                gl_Position = vec4(pos, 0.0, 1.0) * matrix;
                v_color = color;
                v_back_color = back_color;
                v_tex_coord = tex_coord;
            }
            ",

            fragment_shader: "
            #version 150 core
            uniform sampler2D tex;
            in vec4 v_color;
            in vec4 v_back_color;
            in vec2 v_tex_coord;
            out vec4 f_color;

            void main() {
                vec4 tex_color = texture(tex, v_tex_coord);

                // Discard fully transparent pixels to keep them from
                // writing into the depth buffer.
                if (tex_color.a == 0.0) discard;

                f_color = v_color * tex_color + v_back_color * (vec4(1, 1, 1, 1) - tex_color);
            }
            ",
            tessellation_control_shader: None,
            tessellation_evaluation_shader: None,
            geometry_shader: None,
        };

        let inner = backend::Backend::start(width, height, title, SHADER)?;

        Ok(Backend { inner })
    }

    /// Helper method for making a vitral `Core` of the correct type
    pub fn new_core(&mut self) -> Core {
        // Make sure to reuse the existing solid texture so that the Core builder won't do new
        // texture allocations.
        vitral::Builder::new()
            .solid_texture(cache::solid())
            .build(self.inner.canvas_size().cast(), |img| {
                self.inner.make_texture(img)
            })
    }

    /// Return the next keypress event if there is one.
    pub fn poll_key(&mut self) -> Option<KeyEvent> { self.inner.poll_key() }

    /// Display the backend and read input events.
    pub fn update(&mut self, core: &mut Core) -> bool {
        cache::ATLAS.with(|a| self.inner.sync_with_atlas_cache(&mut a.borrow_mut()));
        self.inner.update(core)
    }

    pub fn save_screenshot(&self, basename: &str) -> io::Result<()> {
        calx::save_screenshot(basename, &self.inner.screenshot().into())
    }
}

// Use a custom Vertex with the vitral::Vertex adapter trait because Magog sprites use two color
// parameters while Vitral's draw API only expects one.

#[derive(Copy, Clone)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub color: Color,
    pub back_color: Color,
    pub tex_coord: [f32; 2],
}
implement_vertex!(Vertex, pos, color, back_color, tex_coord);

impl Vertex {
    pub fn new(
        pos: Point2D<f32>,
        tex_coord: Point2D<f32>,
        color: Color,
        back_color: Color,
    ) -> Vertex {
        Vertex {
            pos: [pos.x, pos.y],
            color,
            back_color,
            tex_coord: [tex_coord.x, tex_coord.y],
        }
    }
}

impl vitral::Vertex for Vertex {
    fn new(pos: Point2D<f32>, tex_coord: Point2D<f32>, color: Color) -> Self {
        Vertex {
            pos: [pos.x, pos.y],
            color,
            back_color: [0.0, 0.0, 0.0, 1.0],
            tex_coord: [tex_coord.x, tex_coord.y],
        }
    }
}
