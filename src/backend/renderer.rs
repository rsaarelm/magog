use std::num::{Float};
use std::default::Default;
use image::{ImageBuffer, Rgb};
use glium;
use glium::texture;
use glium::framebuffer;
use glium::render_buffer;
use glium::LinearBlendingFactor::*;
use super::{CanvasMagnify};
use ::{V2, Rect};
use ::rgb::{to_srgb};

pub struct Renderer {
    /// Canvas size.
    size: V2<u32>,
    /// Rendering device resolution.
    resolution: V2<u32>,
    /// Shader for drawing atlas images.
    sprite_shader: glium::Program,
    /// Shader for blitting the canvas texture to screen.
    blit_shader: glium::Program,
    /// Atlas texture, contains all the sprites. Calx is a low-rent operation
    /// so we only have one.
    atlas: texture::Texture2d,
    /// Render target texture.
    buffer: texture::Texture2d,
    params: glium::DrawParameters,
    magnify: CanvasMagnify,
}

impl Renderer {
    pub fn new<'a, T>(
        size: V2<u32>, display: &glium::Display,
        texture_image: T, magnify: CanvasMagnify) -> Renderer
        where T: texture::Texture2dDataSource<'a> {

        let sprite_shader = glium::Program::from_source(display,
            include_str!("sprite.vert"),
            include_str!("sprite.frag"),
            None).unwrap();
        let blit_shader = glium::Program::from_source(display,
            include_str!("blit.vert"),
            include_str!("blit.frag"),
            None).unwrap();
        let atlas = texture::Texture2d::new(display, texture_image);

        let buffer = texture::Texture2d::new_empty(
            display,
            texture::UncompressedFloatFormat::U8U8U8U8,
            size.0, size.1);

        let mut params: glium::DrawParameters = Default::default();
        params.backface_culling = glium::BackfaceCullingMode::CullCounterClockWise;
        params.depth_test = glium::DepthTest::IfLessOrEqual;
        params.depth_write = true;
        params.blending_function = Some(glium::BlendingFunction::Addition {
            source: SourceAlpha, destination: OneMinusSourceAlpha });

        Renderer {
            size: size,
            resolution: size,
            sprite_shader: sprite_shader,
            blit_shader: blit_shader,
            atlas: atlas,
            buffer: buffer,
            params: params,
            magnify: magnify,
        }
    }

    /// Draw sprites on target.
    fn draw_sprites<S>(&self, display: &glium::Display, target: &mut S,
                       vertices: Vec<Vertex>, indices: Vec<u16>)
        where S: glium::Surface {

        // Extract the geometry accumulation buffers and convert into
        // temporary Glium buffers.
        let vertices = glium::VertexBuffer::new(display, vertices);
        let indices = glium::IndexBuffer::new(
            display, glium::index::TrianglesList(indices));

        let uniforms = glium::uniforms::UniformsStorage::new("texture",
            glium::uniforms::Sampler(&self.atlas, glium::uniforms::SamplerBehavior {
                magnify_filter: glium::uniforms::MagnifySamplerFilter::Nearest,
                .. Default::default() }));

        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target.clear_depth(1.0);
        target.draw(&vertices, &indices, &self.sprite_shader, &uniforms, &self.params).unwrap();
    }

    /// Blit the buffer texture to target.
    fn blit_buffer<S>(&self, display: &glium::Display, target: &mut S)
        where S: glium::Surface {
        // TODO: Pixel-perfect scaling to target dimensions.
        //
        let Rect(V2(sx, sy), V2(sw, sh)) = match self.magnify {
            CanvasMagnify::PixelPerfect => pixel_perfect(self.size, self.resolution),
            _ => preserve_aspect(self.size, self.resolution),
        };

        let vertices = {
            #[derive(Copy, Clone)]
            struct BlitVertex { pos: [f32; 2], tex_coord: [f32; 2] }
            implement_vertex!(BlitVertex, pos, tex_coord);

            glium::VertexBuffer::new(display,
            vec![
                BlitVertex { pos: [sx,    sy   ], tex_coord: [0.0, 0.0] },
                BlitVertex { pos: [sx+sw, sy   ], tex_coord: [1.0, 0.0] },
                BlitVertex { pos: [sx+sw, sy+sh], tex_coord: [1.0, 1.0] },
                BlitVertex { pos: [sx,    sy+sh], tex_coord: [0.0, 1.0] },
            ])
        };

        let indices = glium::IndexBuffer::new(display,
            glium::index::TrianglesList(vec![0u16, 1, 2, 0, 2, 3]));

        let mut params: glium::DrawParameters = Default::default();
        // Set an explicit viewport to apply the custom resolution that fixes
        // pixel perfect rounding errors.
        params.viewport = Some(glium::Rect{
            left: 0, bottom: 0,
            width: self.resolution.0,
            height: self.resolution.1 });

        let mag_filter = match self.magnify {
            CanvasMagnify::Smooth => glium::uniforms::MagnifySamplerFilter::Linear,
            _ => glium::uniforms::MagnifySamplerFilter::Nearest
        };

        let uniforms = glium::uniforms::UniformsStorage::new("texture",
            glium::uniforms::Sampler(&self.buffer, glium::uniforms::SamplerBehavior {
                magnify_filter: mag_filter,
                minify_filter: glium::uniforms::MinifySamplerFilter::Linear,
                .. Default::default() }));

        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target.clear_depth(1.0);
        target.draw(&vertices, &indices, &self.blit_shader, &uniforms, &params).unwrap();
    }

    /// Draw a geometry buffer.
    pub fn draw<S>(&mut self, display: &glium::Display, target: &mut S,
                   vertices: Vec<Vertex>, indices: Vec<u16>)
        where S: glium::Surface {

        // Render the graphics to a texture to keep the pixels pure and
        // untainted.
        let depth = render_buffer::DepthRenderBuffer::new(
            display, texture::DepthFormat::F32, self.size.0, self.size.1);
        let mut sprite_target = framebuffer::SimpleFrameBuffer::with_depth_buffer(
            display, &self.buffer, &depth);
        self.draw_sprites(display, &mut sprite_target, vertices, indices);

        let (w, h) = display.get_framebuffer_dimensions();
        // Clip viewport dimensions to even to prevent rounding errors in
        // pixel perfect scaling.
        self.resolution = V2(w & !1, h & !1);
        // Render the texture to screen.
        self.blit_buffer(display, target);
    }

    /// Map screen position (eg. of a mouse cursor) to canvas position.
    pub fn screen_to_canvas(&self, V2(sx, sy): V2<i32>) -> V2<i32> {
        let Rect(V2(rx, ry), V2(rw, rh)) = pixel_perfect(self.size, self.resolution);
        // Transform to device coordinates.
        let sx = sx as f32 * 2.0 / self.resolution.0 as f32 - 1.0;
        let sy = sy as f32 * 2.0 / self.resolution.1 as f32 - 1.0;

        V2(((sx - rx) * self.size.0 as f32 / rw) as i32,
           ((sy - ry) * self.size.1 as f32 / rh) as i32)
    }

    pub fn canvas_pixels(&self) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        let mut ret: ImageBuffer<Rgb<u8>, Vec<u8>> = self.buffer.read();

        // Convert to sRGB
        // XXX: Probably horribly slow, can we make OpenGL do this?
        for p in ret.pixels_mut() {
            p.data[0] = (to_srgb(p.data[0] as f32 / 255.0) * 255.0).round() as u8;
            p.data[1] = (to_srgb(p.data[1] as f32 / 255.0) * 255.0).round() as u8;
            p.data[2] = (to_srgb(p.data[2] as f32 / 255.0) * 255.0).round() as u8;
        }

        ret
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

/// A pixel perfect centered and scaled rectangle of resolution dim in a
/// window of size area, mapped to OpenGL device coordinates.
#[inline(always)]
fn pixel_perfect(canvas: V2<u32>, window: V2<u32>) -> Rect<f32> {
    // Scale based on whichever of X or Y axis is the tighter fit.
    let mut scale = (window.0 as f32 / canvas.0 as f32)
        .min(window.1 as f32 / canvas.1 as f32);

    if scale > 1.0 {
        // Snap to pixel scale if more than 1 window pixel per canvas pixel.
        scale = scale.floor();
    }

    let dim = V2((scale * canvas.0 as f32) * 2.0 / window.0 as f32,
                 (scale * canvas.1 as f32) * 2.0 / window.1 as f32);
    let offset = -dim / 2.0;
    Rect(offset, dim)
}

#[inline(always)]
fn preserve_aspect(canvas: V2<u32>, window: V2<u32>) -> Rect<f32> {
    // Scale based on whichever of X or Y axis is the tighter fit.
    let scale = (window.0 as f32 / canvas.0 as f32)
        .min(window.1 as f32 / canvas.1 as f32);

    let dim = V2((scale * canvas.0 as f32) * 2.0 / window.0 as f32,
                 (scale * canvas.1 as f32) * 2.0 / window.1 as f32);
    let offset = -dim / 2.0;
    Rect(offset, dim)
}
