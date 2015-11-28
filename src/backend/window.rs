use time;
use std::thread;
use glium::{self, glutin, texture, framebuffer, Surface, DisplayBuild};
use image;
use ::{V2, Rect, AverageDuration, color, Rgba};
use super::event::{Event, MouseButton};
use super::{CanvasMagnify};
use super::event_translator::{EventTranslator};

pub struct WindowBuilder {
    title: String,
    size: V2<u32>,
    frame_interval: Option<f64>,
    fullscreen: bool,
    layout_independent_keys: bool,
    magnify: CanvasMagnify,
}

impl WindowBuilder {
    pub fn new() -> WindowBuilder {
        WindowBuilder {
            title: "".to_string(),
            size: V2(640, 360),
            frame_interval: None,
            fullscreen: false,
            layout_independent_keys: true,
            magnify: CanvasMagnify::PixelPerfect,
        }
    }

    /// Set the window title.
    pub fn set_title(mut self, title: &str) -> WindowBuilder {
        self.title = title.to_string();
        self
    }

    /// Set the frame rate.
    pub fn set_frame_interval(mut self, interval_s: f64) -> WindowBuilder {
        assert!(interval_s > 0.00001);
        self.frame_interval = Some(interval_s);
        self
    }

    /// Set the size of the logical canvas.
    pub fn set_size(mut self, width: u32, height: u32) -> WindowBuilder {
        self.size = V2(width, height);
        self
    }

    /// Get the key values from the user's keyboard layout instead of the
    /// hardware keyboard map. Hardware keymap lookup may not work correctly
    /// on all platforms.
    pub fn use_layout_dependent_keys(mut self) -> WindowBuilder {
        self.layout_independent_keys = false;
        self
    }

    /// Set the canvas to start in fullscreen mode.
    /// FIXME: Broken on Linux, https://github.com/tomaka/glutin/issues/148
    pub fn set_fullscreen(mut self, state: bool) -> WindowBuilder {
        self.fullscreen = state;
        self
    }

    pub fn set_magnify(mut self, magnify: CanvasMagnify) -> WindowBuilder {
        self.magnify = magnify;
        self
    }

    /// Build the window object.
    pub fn build(self) -> Window {
        Window::new(self)
    }
}

/// Toplevel application object.
pub struct Window {
    pub display: glium::Display,
    pub clear_color: Rgba,

    resolution: LogicalResolution,

    /// Shader for blitting the canvas texture to screen.
    blit_shader: glium::Program,

    /// Render target texture.
    buffer: texture::Texture2d,
    depth: framebuffer::DepthRenderBuffer,

    translator: EventTranslator,
    frame_duration: AverageDuration,

    frame_interval: Option<f64>,
    previous_frame_t: f64,
}

impl Window {
    fn new(builder: WindowBuilder) -> Window {
        use glium::glutin::{GlRequest, Api};
        let size = builder.size;
        let title = &builder.title[..];

        let mut glutin = glutin::WindowBuilder::new()
            .with_gl(GlRequest::Specific(Api::OpenGl, (3, 2)))
            .with_title(title.to_string());

        if builder.fullscreen {
            // FIXME: Glutin's X11 fullscreen is broken, this is only enabled
            // for Windows.
            if cfg!(windows) {
                glutin = glutin.with_fullscreen(glutin::get_primary_monitor());
            }
        } else {
            // Zoom up the window to the biggest even pixel multiple that fits
            // the user's monitor.
            let window_border_guesstimate = 32;
            let (w, h) = glutin::get_primary_monitor().get_dimensions();
            let window_size = V2(w, h) - V2(window_border_guesstimate, window_border_guesstimate);

            let (mut x, mut y) = (size.0, size.1);
            while x * 2 <= window_size.0 && y * 2 <= window_size.1 {
                x *= 2;
                y *= 2;
            }

            glutin = glutin.with_dimensions(x, y);
        }

        let display = glutin.build_glium().unwrap();

        let (w, h) = display.get_framebuffer_dimensions();

        let resolution = LogicalResolution::new(builder.magnify, size, V2(w, h));

        let blit_shader = glium::Program::from_source(&display,
            include_str!("blit.vert"),
            include_str!("blit.frag"),
            None).unwrap();

        let buffer = texture::Texture2d::empty(
            &display, size.0, size.1).unwrap();
        let depth = framebuffer::DepthRenderBuffer::new(
            &display, texture::DepthFormat::F32, size.0, size.1).unwrap();

        Window {
            display: display,
            clear_color: color::BLACK,
            resolution: resolution,
            blit_shader: blit_shader,
            buffer: buffer,
            depth: depth,
            translator: EventTranslator::new(builder.layout_independent_keys),
            frame_duration: AverageDuration::new(0.1, 0.9),
            frame_interval: builder.frame_interval,
            previous_frame_t: time::precise_time_s(),
        }
    }

    /// Display the screen buffer and do end-of-frame bookkeeping.
    pub fn end_frame(&mut self) {
        self.frame_duration.tick();
        self.show();

        // Stick to a target frame rate if one is set.
        if let Some(target_t) = self.frame_interval {
            let sleepytime = target_t - (time::precise_time_s() - self.previous_frame_t);
            if sleepytime > 0.0 { thread::sleep_ms((sleepytime * 1e3) as u32); }
        }
        self.previous_frame_t = time::precise_time_s();

        // Pull in new input events. This will sleep if there was a suspend
        // event.
        self.translator.pump(&mut self.display, &self.resolution);
    }

    pub fn draw<F>(&self, draw_f: F)
        where F: FnOnce(&mut framebuffer::SimpleFrameBuffer) {
        let mut target = framebuffer::SimpleFrameBuffer::with_depth_buffer(
            &self.display, &self.buffer, &self.depth).unwrap();
        draw_f(&mut target);
    }

    /// Fill the frame with the given pixel buffer. The pixel data dimensions
    /// must match the logical size of the window.
    pub fn set_frame<'a, T: glium::texture::Texture2dDataSource<'a>>(&mut self, pixels: T) {
        let new_texture = texture::Texture2d::new(&self.display, pixels).unwrap();
        assert!(new_texture.get_width() == self.resolution.canvas.0 &&
                new_texture.get_height() == Some(self.resolution.canvas.1),
            "Pixel data dimensions do not match logical window size");
        self.buffer = new_texture;
    }

    /// Return the logical window resolution.
    #[inline(always)]
    pub fn size(&self) -> V2<u32> { self.resolution.canvas }

    pub fn mouse_pos(&self) -> V2<f32> { self.translator.mouse_pos }

    pub fn mouse_pressed(&self, button: MouseButton) -> bool {
        self.translator.mouse_pressed[button as usize].is_some()
    }

    /// Return the current exponential moving average for frame rendering
    /// duration in seconds.
    pub fn frame_duration(&self) -> f64 { self.frame_duration.value }

    pub fn events(&mut self) -> Vec<Event> {
        // XXX: Not returning an iterator because of locking crap. May want to
        // do mutable stuff on window in response to events.
        EventIterator { window: self }.collect::<Vec<Event>>()
    }

    /// Map screen position (eg. of a mouse cursor) to canvas position.
    pub fn screen_to_canvas(&self, screen_pos: V2<i32>) -> V2<i32> {
        self.resolution.screen_to_canvas(screen_pos)
    }

    pub fn get_screenshot(&self) -> image::ImageBuffer<image::Rgb<u8>, Vec<u8>> {
        use ::rgb::{to_srgb};
        let mut ret = self.buffer.read::<image::DynamicImage>().to_rgb();

        // Convert to sRGB
        // XXX: Probably horribly slow, can we make OpenGL do this?
        for p in ret.pixels_mut() {
            p.data[0] = (to_srgb(p.data[0] as f32 / 255.0) * 255.0).round() as u8;
            p.data[1] = (to_srgb(p.data[1] as f32 / 255.0) * 255.0).round() as u8;
            p.data[2] = (to_srgb(p.data[2] as f32 / 255.0) * 255.0).round() as u8;
        }

        ret
    }

    /// Show the graphics buffer on screen.
    fn show(&mut self) {
        let mut target = self.display.draw();

        // Clear the window.
        target.clear_color(
            self.clear_color.r,
            self.clear_color.g,
            self.clear_color.b,
            self.clear_color.a);
        target.clear_depth(1.0);
        let (w, h) = self.display.get_framebuffer_dimensions();
        // Clip viewport dimensions to even to prevent rounding errors in
        // pixel perfect scaling.
        self.resolution.window = V2(w & !1, h & !1);

        // Build the geometry for the on-screen rectangle.
        let Rect(V2(sx, sy), V2(sw, sh)) = self.resolution.screen_rect();

        // XXX: This could use glium::Surface::blit_whole_color_to instead of
        // the handmade blitting, but that was buggy on Windows around
        // 2015-03.

        let vertices = {
            #[derive(Copy, Clone)]
            struct BlitVertex { pos: [f32; 2], tex_coord: [f32; 2] }
            implement_vertex!(BlitVertex, pos, tex_coord);

            glium::VertexBuffer::new(&self.display,
            &[
                BlitVertex { pos: [sx,    sy   ], tex_coord: [0.0, 0.0] },
                BlitVertex { pos: [sx+sw, sy   ], tex_coord: [1.0, 0.0] },
                BlitVertex { pos: [sx+sw, sy+sh], tex_coord: [1.0, 1.0] },
                BlitVertex { pos: [sx,    sy+sh], tex_coord: [0.0, 1.0] },
            ]).unwrap()
        };

        let indices = glium::IndexBuffer::new(&self.display,
            glium::index::PrimitiveType::TrianglesList, &[0u16, 1, 2, 0, 2, 3]).unwrap();

        // Set up the rest of the draw parameters.
        let mut params: glium::DrawParameters = Default::default();
        // Set an explicit viewport to apply the custom resolution that fixes
        // pixel perfect rounding errors.
        params.viewport = Some(glium::Rect{
            left: 0, bottom: 0,
            width: self.resolution.window.0,
            height: self.resolution.window.1 });

        let mag_filter = match self.resolution.magnify {
            CanvasMagnify::Smooth => glium::uniforms::MagnifySamplerFilter::Linear,
            _ => glium::uniforms::MagnifySamplerFilter::Nearest
        };

        let uniforms = glium::uniforms::UniformsStorage::new("tex",
            glium::uniforms::Sampler(&self.buffer, glium::uniforms::SamplerBehavior {
                magnify_filter: mag_filter,
                minify_filter: glium::uniforms::MinifySamplerFilter::Linear,
                .. Default::default() }));

        // Draw the graphics buffer to the window.
        target.draw(&vertices, &indices, &self.blit_shader, &uniforms, &params).unwrap();
        target.finish().unwrap();
    }
}

pub struct LogicalResolution {
    pub magnify: CanvasMagnify,
    /// Logical size, pixel on virtual canvas.
    pub canvas: V2<u32>,
    /// Physical size, window on screen.
    pub window: V2<u32>,
}

impl LogicalResolution {
    pub fn new(magnify: CanvasMagnify, canvas_size: V2<u32>, resolution: V2<u32>) -> LogicalResolution {
        LogicalResolution {
            magnify: magnify,
            canvas: canvas_size,
            window: resolution,
        }
    }

    pub fn screen_rect(&self) -> Rect<f32> {
        match self.magnify {
            CanvasMagnify::PixelPerfect => self.pixel_perfect(),
            _ => self.preserve_aspect(),
        }
    }

    /// Map screen position (eg. of a mouse cursor) to canvas position.
    pub fn screen_to_canvas(&self, V2(sx, sy): V2<i32>) -> V2<i32> {
        let Rect(V2(rx, ry), V2(rw, rh)) = self.screen_rect();

        // Transform to device coordinates.
        let sx = sx as f32 * 2.0 / self.window.0 as f32 - 1.0;
        let sy = sy as f32 * 2.0 / self.window.1 as f32 - 1.0;

        V2(((sx - rx) * self.canvas.0 as f32 / rw) as i32,
           ((sy - ry) * self.canvas.1 as f32 / rh) as i32)
    }

    /// A pixel perfect centered and scaled rectangle of resolution dim in a
    /// window of size area, mapped to OpenGL device coordinates.
#[inline(always)]
    fn pixel_perfect(&self) -> Rect<f32> {
        // Scale based on whichever of X or Y axis is the tighter fit.
        let mut scale = (self.window.0 as f32 / self.canvas.0 as f32)
            .min(self.window.1 as f32 / self.canvas.1 as f32);

        if scale > 1.0 {
            // Snap to pixel scale if more than 1 window pixel per canvas pixel.
            scale = scale.floor();
        }

        let dim = V2((scale * self.canvas.0 as f32) * 2.0 / self.window.0 as f32,
        (scale * self.canvas.1 as f32) * 2.0 / self.window.1 as f32);
        let offset = -dim / 2.0;
        Rect(offset, dim)
    }

#[inline(always)]
    fn preserve_aspect(&self) -> Rect<f32> {
        // Scale based on whichever of X or Y axis is the tighter fit.
        let scale = (self.window.0 as f32 / self.canvas.0 as f32)
            .min(self.window.1 as f32 / self.canvas.1 as f32);

        let dim = V2((scale * self.canvas.0 as f32) * 2.0 / self.window.0 as f32,
        (scale * self.canvas.1 as f32) * 2.0 / self.window.1 as f32);
        let offset = -dim / 2.0;
        Rect(offset, dim)
    }
}

pub struct EventIterator<'a> {
    window: &'a mut Window
}

impl<'a> Iterator for EventIterator<'a> {
    type Item = Event;
    fn next(&mut self) -> Option<Event> { self.window.translator.next() }
}
