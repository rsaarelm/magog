use time;
use std::mem;
use std::num::{FloatMath, Float};
use image::{GenericImage, SubImage, Pixel};
use image::{ImageBuffer, Rgba};
use image;
use glutin;
use glium::{DisplayBuild};
use gfx;
use atlas::{AtlasBuilder, Atlas};
use util;
use geom::{V2, Rect};
use event::Event;
use rgb::Rgb;
use renderer::{Renderer, Vertex};
use super::{Color};
use scancode;

pub static FONT_W: uint = 8;
pub static FONT_H: uint = 8;

/// The first font image in the atlas image set.
static FONT_IDX: uint = 0;

/// Image index of the solid color texture block.
static SOLID_IDX: uint = 96;

pub struct Canvas {
    title: String,
    dim: V2<u32>,
    frame_interval: Option<f64>,
    builder: AtlasBuilder,
}

/// Toplevel graphics drawing and input reading context.
impl Canvas {
    pub fn new() -> Canvas {
        let mut ret = Canvas {
            title: "window".to_string(),
            dim: V2(640, 360),
            frame_interval: None,
            builder: AtlasBuilder::new(),
        };
        ret.init_font();
        ret.init_solid();
        ret
    }

    pub fn set_title(mut self, title: &str) -> Canvas {
        self.title = title.to_string();
        self
    }

    /// Set the frame rate.
    pub fn set_frame_interval(mut self, interval_s: f64) -> Canvas {
        assert!(interval_s > 0.00001);
        self.frame_interval = Some(interval_s);
        self
    }

    /// Set the resolution.
    pub fn set_dim(mut self, dim: V2<u32>) -> Canvas {
        self.dim = dim;
        self
    }

    pub fn add_image<P: Pixel<u8> + 'static, I: GenericImage<P>>(
        &mut self, offset: V2<int>, image: I) -> Image {
        Image(self.builder.push(offset, image))
    }

    /// Start running the engine, return an event iteration.
    pub fn run(&mut self) -> Context {
        Context::new(
            self.dim,
            self.title.as_slice(),
            self.frame_interval,
            Atlas::new(&self.builder))
    }

    /// Load the default font into the texture atlas.
    fn init_font(&mut self) {
        let mut font_sheet = util::color_key(
            &image::load_from_memory(include_bytes!("../assets/font.png"), image::PNG).unwrap(),
            &Rgb::new(0x80u8, 0x80u8, 0x80u8));
        for i in range(0u32, 96u32) {
            let x = 8u32 * (i % 16u32);
            let y = 8u32 * (i / 16u32);
            let Image(idx) = self.add_image(V2(0, -8), SubImage::new(&mut font_sheet, x, y, 8, 8));
            assert!(idx - i as uint == FONT_IDX);
        }
    }

    /// Load a solid color element into the texture atlas.
    fn init_solid(&mut self) {
        let image: ImageBuffer<Vec<u8>, u8, Rgba<u8>> = ImageBuffer::from_fn(1, 1, |_, _| Rgba([0xffu8, 0xffu8, 0xffu8, 0xffu8]));
        let Image(idx) = self.add_image(V2(0, 0), image);
        assert!(idx == SOLID_IDX);
    }
}

/// Interface to render to a live display.
pub struct Context {
    window: glutin::Window,
    events: Vec<glutin::Event>,
    renderer: Renderer,

    atlas: Atlas,
    triangle_buf: Vec<Vertex>,

    state: State,
    frame_interval: Option<f64>,
    last_render_time: f64,
    image_dims: Vec<V2<uint>>,
    resolution: V2<u32>,
    window_resolution: V2<u32>,

    /// Time in seconds it took to render the last frame.
    pub render_duration: f64,
}

#[deriving(PartialEq)]
enum State {
    Normal,
    EndFrame,
}

impl Context {
    fn new(
        dim: V2<u32>,
        title: &str,
        frame_interval: Option<f64>,
        atlas: Atlas) -> Context {

        let window = glutin::WindowBuilder::new()
            .with_title(title.to_string())
            .with_dimensions(dim.0 as uint, dim.1 as uint)
            .build().unwrap();

        unsafe { window.make_current(); }

        let (w, h) = window.get_inner_size().unwrap();

        let renderer = Renderer::new();

        let mut dims = vec![];

        for i in range(0, atlas.vertices.len()) {
            dims.push(atlas.vertices[i].1.map(|x| x as uint));
        }

        Context {
            window: window,
            events: Vec::new(),
            renderer: renderer,

            atlas: atlas,
            triangle_buf: Vec::new(),

            state: State::Normal,
            frame_interval: frame_interval,
            last_render_time: time::precise_time_s(),
            image_dims: dims,
            resolution: dim,
            window_resolution: V2(w as u32, h as u32),

            render_duration: 0.1f64,
        }
    }

    /// Clear the screen
    pub fn clear<C: Color>(&mut self, color: &C) {
        self.renderer.clear(color);
        self.triangle_buf.clear();
    }

    fn window_to_device(&self, window_pos: V2<int>, z: f32) -> [f32, ..3] {
        let V2(w, h) = self.window_resolution;
        let Rect(V2(rx, ry), V2(rw, _)) = pixel_perfect(self.resolution, self.window_resolution);
        let zoom = (rw as f32) / (self.resolution.0 as f32);
        [-1.0 + (2.0 * (rx as f32 + window_pos.0 as f32 * zoom) / w as f32),
          1.0 - (2.0 * (ry as f32 + window_pos.1 as f32 * zoom) / h as f32),
         z]
    }

    fn tri_vtx(&mut self, window_pos: V2<int>, layer: f32, texture_pos: V2<f32>, color: [f32, ..4]) {
        let pos = self.window_to_device(window_pos, layer);
        self.triangle_buf.push(Vertex {
            pos: pos,
            color: color,
            tex_coord: texture_pos.to_array() })
    }

    pub fn draw_image<C: Color>(&mut self, offset: V2<int>, layer: f32, Image(idx): Image, color: &C) {
        let color = color.to_rgba();
        let rect = self.atlas.vertices[idx] + offset;
        let tex = self.atlas.texcoords[idx];

        self.tri_vtx(rect.p0(), layer, tex.p0(), color);
        self.tri_vtx(rect.p1(), layer, tex.p1(), color);
        self.tri_vtx(rect.p2(), layer, tex.p2(), color);
        self.tri_vtx(rect.p0(), layer, tex.p0(), color);
        self.tri_vtx(rect.p2(), layer, tex.p2(), color);
        self.tri_vtx(rect.p3(), layer, tex.p3(), color);
    }

    pub fn draw_tri<C: Color>(&mut self, layer: f32, p: [V2<int>, ..3], c: [C, ..3]) {
        let tex = self.atlas.texcoords[SOLID_IDX].0;
        for i in range(0, 3) { self.tri_vtx(p[i], layer, tex, c[i].to_rgba()); }
    }

    pub fn font_image(&self, c: char) -> Option<Image> {
        let idx = c as int;
        // Hardcoded limiting of the font to printable ASCII.
        if idx >= 32 && idx < 128 {
            Some(Image((idx - 32) as uint + FONT_IDX))
        } else {
            None
        }
    }

    pub fn image_dim(&self, Image(idx): Image) -> V2<uint> {
        self.image_dims[idx]
    }
}

impl<'a> Iterator<Event<'a>> for Context {
    fn next(&mut self) -> Option<Event<'a>> {
        // After a render event, control will return here on a new
        // iter call. Do post-render work here.
        if self.state == State::EndFrame {
            self.state = State::Normal;

            self.renderer.draw_triangles(self.triangle_buf.as_slice());

            self.renderer.end_frame();
            self.window.swap_buffers();

            self.triangle_buf.clear();
        }

        loop {
            if self.window.is_closed() {
                return None;
            }

            // XXX: Will aborting the iteration with return keep the remaining
            // events in the queue?
            self.events.push_all(self.window.poll_events().collect::<Vec<glutin::Event>>().as_slice());

            match self.events.remove(0) {
                Some(glutin::Event::ReceivedCharacter(ch)) => {
                    return Some(Event::Char(ch));
                }
                Some(glutin::Event::KeyboardInput(action, scan, _vko)) => {
                    if (scan as uint) < scancode::MAP.len() {
                        if let Some(key) = scancode::MAP[scan as uint] {
                            return Some(if action == glutin::ElementState::Pressed {
                                Event::KeyPressed(key)
                            }
                            else {
                                Event::KeyReleased(key)
                            });
                        }
                    }
                }
                // TODO Mouse events.
                _ => ()
            }

            let t = time::precise_time_s();
            if self.frame_interval.map_or(true,
                |x| t - self.last_render_time >= x) {
                let delta = t - self.last_render_time;
                let sensitivity = 0.25f64;
                self.render_duration = (1f64 - sensitivity) * self.render_duration + sensitivity * delta;

                self.last_render_time = t;

                // Time to render, must return a handle to self.
                // XXX: Need unsafe hackery to get around lifetimes check.
                self.state = State::EndFrame;

                let (w, h) = self.window.get_inner_size().unwrap();
                self.window_resolution = V2(w as u32, h as u32);
                self.renderer.set_window_size((w as i32, h as i32));
                self.renderer.scissor(pixel_perfect(self.resolution, self.window_resolution));

                unsafe {
                    return Some(Event::Render(mem::transmute(self)))
                }
            }
        }
    }
}

/// Drawable images stored in the Canvas.
#[deriving(Copy, Clone, PartialEq)]
pub struct Image(uint);

/// A pixel perfect centered and scaled rectangle of resolution dim in a
/// window of size area.
fn pixel_perfect(canvas: V2<u32>, window: V2<u32>) -> Rect<u32> {
    let mut scale = (window.0 as f32 / canvas.0 as f32)
        .min(window.1 as f32 / canvas.1 as f32);

    if scale > 1.0 {
        // Snap to pixel scale if more than 1 window pixel per canvas pixel.
        scale = scale.floor();
    }

    let dim = V2((scale * canvas.0 as f32) as u32, (scale * canvas.1 as f32) as u32);
    let offset = (window - dim) / 2;
    Rect(offset, dim)
}
