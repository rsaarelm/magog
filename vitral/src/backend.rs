//! Glium-based backend for the Vitral GUI library.

#![deny(missing_docs)]

use crate::atlas_cache::AtlasCache;
use crate::canvas_zoom::CanvasZoom;
use crate::{DrawBatch, TextureIndex, Vertex};
use euclid::default::Size2D;
use glium::glutin;
use glium::index::PrimitiveType;
use glium::{self, implement_vertex, program, uniform, Surface};
use image::{Pixel, RgbImage, RgbaImage};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use std::error::Error;
use std::fmt::Debug;
use std::hash::Hash;

/// Default texture type used by the backend.
type GliumTexture = glium::texture::SrgbTexture2d;

/// Glium-rendering backend for Vitral.
pub struct Backend {
    pub display: glium::Display,
    program: glium::Program,
    textures: Vec<GliumTexture>,
    pub render_buffer: RenderBuffer,
    pub zoom: CanvasZoom,
    pub window_size: Size2D<u32>,
}

impl Backend {
    /// Create a new Glium backend for Vitral.
    ///
    /// The backend requires an user-supplied vertex type as a type parameter and a shader program
    /// to render data of that type as argument to the constructor.
    fn new(display: glium::Display, program: glium::Program, width: u32, height: u32) -> Backend {
        let (w, h) = get_size(&display);
        let render_buffer = RenderBuffer::new(&display, width, height);

        Backend {
            display,
            program,
            textures: Vec::new(),
            render_buffer,
            zoom: CanvasZoom::PixelPerfect,
            window_size: Size2D::new(w, h),
        }
    }

    /// Open a Glium window and start a backend for it.
    pub fn start<T>(
        event_loop: &EventLoop<T>,
        window: WindowBuilder,
        width: u32,
        height: u32,
        pixel_perfect: bool,
    ) -> Result<Backend, Box<dyn Error>> {
        let context = glutin::ContextBuilder::new()
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 2)));
        let display = glium::Display::new(window, context, event_loop)?;
        let program = glium::Program::new(&display, DEFAULT_SHADER)?;

        let mut ret = Backend::new(display, program, width, height);
        if !pixel_perfect {
            ret.zoom = CanvasZoom::AspectPreserving;
        }

        Ok(ret)
    }

    /// Return the pixel resolution of the backend.
    ///
    /// Note that this is the logical size which will stay the same even when the
    /// desktop window is resized.
    pub fn canvas_size(&self) -> Size2D<u32> { self.render_buffer.size }

    /// Return the current number of textures.
    pub fn texture_count(&self) -> usize { self.textures.len() }

    /// Make a new empty internal texture.
    ///
    /// The new `TextureIndex` must equal the value `self.texture_count()` would have returned
    /// just before calling this.
    pub fn make_empty_texture(&mut self, width: u32, height: u32) -> TextureIndex {
        let tex = glium::texture::SrgbTexture2d::empty(&self.display, width, height).unwrap();
        self.textures.push(tex);
        self.textures.len() - 1
    }

    /// Rewrite an internal texture.
    pub fn write_to_texture(&mut self, img: RgbaImage, texture: TextureIndex) {
        assert!(
            texture < self.textures.len(),
            "Trying to write nonexistent texture"
        );
        let (width, height) = img.dimensions();
        let rect = glium::Rect {
            left: 0,
            bottom: 0,
            width,
            height,
        };
        let mut raw = glium::texture::RawImage2d::from_raw_rgba(img.into_raw(), (width, height));
        raw.format = glium::texture::ClientFormat::U8U8U8U8;

        self.textures[texture].write(rect, raw);
    }

    /// Make a new internal texture using image data.
    pub fn make_texture(&mut self, img: RgbaImage) -> TextureIndex {
        let size = img.dimensions();
        let mut raw = glium::texture::RawImage2d::from_raw_rgba(img.into_raw(), size);
        raw.format = glium::texture::ClientFormat::U8U8U8U8;

        let tex = glium::texture::SrgbTexture2d::new(&self.display, raw).unwrap();
        self.textures.push(tex);
        self.textures.len() - 1
    }

    /// Update or construct textures based on changes in atlas cache.
    pub fn sync_with_atlas_cache<T: Eq + Hash + Clone + Debug>(
        &mut self,
        atlas_cache: &mut AtlasCache<T>,
    ) {
        for a in atlas_cache.atlases_mut() {
            let idx = a.texture();
            // If there are sheets in the atlas that don't have corresponding textures yet,
            // construct those now.
            while idx >= self.texture_count() {
                self.make_empty_texture(a.size().width, a.size().height);
            }

            // Write the updated texture atlas to internal texture.
            a.update_texture(|buf, idx| self.write_to_texture(buf.clone(), idx));
        }
    }

    /// Render draw list from canvas into the frame buffer.
    fn render_list(&mut self, draw_list: &[DrawBatch]) {
        let mut target = self.render_buffer.get_framebuffer_target(&self.display);
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        let (w, h) = target.get_dimensions();

        for batch in draw_list {
            // building the uniforms
            let uniforms = uniform! {
                matrix: [
                    [2.0 / w as f32, 0.0, 0.0, -1.0],
                    [0.0, -2.0 / h as f32, 0.0, 1.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0f32]
                ],
                tex: glium::uniforms::Sampler::new(&self.textures[batch.texture])
                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
            };

            let vertex_buffer =
                { glium::VertexBuffer::new(&self.display, &batch.vertices).unwrap() };

            // building the index buffer
            let index_buffer = glium::IndexBuffer::new(
                &self.display,
                PrimitiveType::TrianglesList,
                &batch.triangle_indices,
            )
            .unwrap();

            let params = glium::draw_parameters::DrawParameters {
                scissor: batch.clip.map(|clip| glium::Rect {
                    left: clip.origin.x as u32,
                    bottom: h - (clip.origin.y + clip.size.height) as u32,
                    width: clip.size.width as u32,
                    height: clip.size.height as u32,
                }),
                blend: glium::Blend::alpha_blending(),
                ..Default::default()
            };

            target
                .draw(
                    &vertex_buffer,
                    &index_buffer,
                    &self.program,
                    &uniforms,
                    &params,
                )
                .unwrap();
        }
    }

    fn update_window_size(&mut self) {
        let (w, h) = get_size(&self.display);
        self.window_size = Size2D::new(w, h);
    }

    /// Display the backend and read input events.
    pub fn render(&mut self, draw_list: &[DrawBatch]) {
        self.update_window_size();
        self.render_list(draw_list);
        self.render_buffer.draw(&self.display, self.zoom);
    }

    /// Return an image for the current contents of the screen.
    pub fn screenshot(&self) -> RgbImage { self.render_buffer.screenshot() }
}

/// Shader for two parametrizable colors and discarding fully transparent pixels
const DEFAULT_SHADER: glium::program::SourceCode<'_> = glium::program::SourceCode {
    vertex_shader: "
        #version 150 core

        uniform mat4 matrix;

        in vec2 pos;
        in vec2 tex_coord;
        in vec4 color;
        in vec4 back_color;

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
        in vec2 v_tex_coord;
        in vec4 v_back_color;
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

implement_vertex!(Vertex, pos, tex_coord, color, back_color);

/// A deferred rendering buffer for pixel-perfect display.
pub struct RenderBuffer {
    size: Size2D<u32>,
    buffer: glium::texture::SrgbTexture2d,
    depth_buffer: glium::framebuffer::DepthRenderBuffer,
    shader: glium::Program,
}

impl RenderBuffer {
    pub fn new(display: &glium::Display, width: u32, height: u32) -> RenderBuffer {
        let shader = program!(
            display,
            150 => {
            vertex: "
                #version 150 core

                in vec2 pos;
                in vec2 tex_coord;

                out vec2 v_tex_coord;

                void main() {
                    v_tex_coord = tex_coord;
                    gl_Position = vec4(pos, 0.0, 1.0);
                }",
            fragment: "
                #version 150 core

                uniform sampler2D tex;
                in vec2 v_tex_coord;

                out vec4 f_color;

                void main() {
                    vec4 tex_color = texture(tex, v_tex_coord);
                    tex_color.a = 1.0;
                    f_color = tex_color;
                }"})
        .unwrap();

        let buffer = glium::texture::SrgbTexture2d::empty(display, width, height).unwrap();

        let depth_buffer = glium::framebuffer::DepthRenderBuffer::new(
            display,
            glium::texture::DepthFormat::F32,
            width,
            height,
        )
        .unwrap();

        RenderBuffer {
            size: Size2D::new(width, height),
            buffer,
            depth_buffer,
            shader,
        }
    }

    /// Get the render target to the pixel-perfect framebuffer.
    pub fn get_framebuffer_target(
        &mut self,
        display: &glium::Display,
    ) -> glium::framebuffer::SimpleFrameBuffer<'_> {
        glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(
            display,
            &self.buffer,
            &self.depth_buffer,
        )
        .unwrap()
    }

    pub fn draw(&mut self, display: &glium::Display, zoom: CanvasZoom) {
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);

        let (w, h) = get_size(display);

        // Build the geometry for the on-screen rectangle.
        let s_rect = zoom.fit_canvas(Size2D::new(w, h), self.size);

        let (sx, sy) = (s_rect.origin.x, s_rect.origin.y);
        let (sw, sh) = (s_rect.size.width, s_rect.size.height);

        // XXX: This could use glium::Surface::blit_whole_color_to instead of
        // the handmade blitting, but that was buggy on Windows around
        // 2015-03.

        let vertices = {
            #[derive(Copy, Clone)]
            struct BlitVertex {
                pos: [f32; 2],
                tex_coord: [f32; 2],
            }
            implement_vertex!(BlitVertex, pos, tex_coord);

            glium::VertexBuffer::new(
                display,
                &[
                    BlitVertex {
                        pos: [sx, sy],
                        tex_coord: [0.0, 0.0],
                    },
                    BlitVertex {
                        pos: [sx + sw, sy],
                        tex_coord: [1.0, 0.0],
                    },
                    BlitVertex {
                        pos: [sx + sw, sy + sh],
                        tex_coord: [1.0, 1.0],
                    },
                    BlitVertex {
                        pos: [sx, sy + sh],
                        tex_coord: [0.0, 1.0],
                    },
                ],
            )
            .unwrap()
        };

        let indices = glium::IndexBuffer::new(
            display,
            glium::index::PrimitiveType::TrianglesList,
            &[0u16, 1, 2, 0, 2, 3],
        )
        .unwrap();

        // Set up the rest of the draw parameters.
        let mut params: glium::DrawParameters<'_> = Default::default();
        // Set an explicit viewport to apply the custom resolution that fixes
        // pixel perfect rounding errors.
        params.viewport = Some(glium::Rect {
            left: 0,
            bottom: 0,
            width: w,
            height: h,
        });

        // TODO: Option to use smooth filter & non-pixel-perfect scaling
        let mag_filter = glium::uniforms::MagnifySamplerFilter::Nearest;

        let uniforms = glium::uniforms::UniformsStorage::new(
            "tex",
            glium::uniforms::Sampler(
                &self.buffer,
                glium::uniforms::SamplerBehavior {
                    magnify_filter: mag_filter,
                    minify_filter: glium::uniforms::MinifySamplerFilter::Linear,
                    ..Default::default()
                },
            ),
        );

        // Draw the graphics buffer to the window.
        target
            .draw(&vertices, &indices, &self.shader, &uniforms, &params)
            .unwrap();
        target.finish().unwrap();
    }

    pub fn size(&self) -> Size2D<u32> { self.size }

    pub fn screenshot(&self) -> RgbImage {
        let image: glium::texture::RawImage2d<'_, u8> = self.buffer.read();

        RgbImage::from_fn(image.width, image.height, |x, y| {
            let i = (x * 4 + (image.height - y - 1) * image.width * 4) as usize;
            Pixel::from_channels(image.data[i], image.data[i + 1], image.data[i + 2], 0xff)
        })
    }
}

fn get_size(display: &glium::Display) -> (u32, u32) {
    let size = display
        .gl_window()
        .window()
        .inner_size()
        .to_physical(display.gl_window().window().hidpi_factor());

    (size.width as u32, size.height as u32)
}
