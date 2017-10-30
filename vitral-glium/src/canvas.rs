use canvas_zoom::CanvasZoom;
use euclid::Size2D;
use glium::{self, framebuffer, texture};
use glium::Surface;

/// A deferred rendering buffer for pixel-perfect display.
pub struct Canvas {
    size: Size2D<u32>,
    buffer: texture::SrgbTexture2d,
    depth_buffer: framebuffer::DepthRenderBuffer,
    shader: glium::Program,
}

impl Canvas {
    pub fn new(display: &glium::Display, width: u32, height: u32) -> Canvas {
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
                }"}).unwrap();

        let buffer = texture::SrgbTexture2d::empty(display, width, height).unwrap();

        let depth_buffer =
            framebuffer::DepthRenderBuffer::new(display, texture::DepthFormat::F32, width, height)
                .unwrap();

        Canvas {
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
    ) -> glium::framebuffer::SimpleFrameBuffer {
        framebuffer::SimpleFrameBuffer::with_depth_buffer(display, &self.buffer, &self.depth_buffer)
            .unwrap()
    }

    pub fn draw(&mut self, display: &glium::Display, zoom: CanvasZoom) {
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);

        let (w, h) = display.get_framebuffer_dimensions();

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
            ).unwrap()
        };

        let indices = glium::IndexBuffer::new(
            display,
            glium::index::PrimitiveType::TrianglesList,
            &[0u16, 1, 2, 0, 2, 3],
        ).unwrap();

        // Set up the rest of the draw parameters.
        let mut params: glium::DrawParameters = Default::default();
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
}
