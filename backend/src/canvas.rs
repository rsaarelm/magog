use time;
use std::mem;
use std::num::{Float};
use image::{GenericImage, SubImage, Pixel};
use image::{ImageBuffer, Rgba};
use image;
use glutin;
use glium::{self, DisplayBuild};
use util::{self, AtlasBuilder, Atlas, AtlasItem, V2, Rect, Rgb, Color};
use event::{Event, MouseButton};
use renderer::{Renderer, Vertex};
use scancode;

pub static FONT_W: u32 = 8;
pub static FONT_H: u32 = 8;

/// The first font image in the atlas image set.
static FONT_IDX: usize = 0;

/// Image index of the solid color texture block.
static SOLID_IDX: usize = 96;

pub struct CanvasBuilder {
    title: String,
    dim: V2<i32>,
    frame_interval: Option<f64>,
    builder: AtlasBuilder,
}

/// Toplevel graphics drawing and input reading context.
impl CanvasBuilder {
    pub fn new() -> CanvasBuilder {
        let mut ret = CanvasBuilder {
            title: "".to_string(),
            dim: V2(640, 360),
            frame_interval: None,
            builder: AtlasBuilder::new(),
        };
        ret.init_font();
        ret.init_solid();
        ret
    }

    /// Set the window title.
    pub fn set_title(mut self, title: &str) -> CanvasBuilder {
        self.title = title.to_string();
        self
    }

    /// Set the frame rate.
    pub fn set_frame_interval(mut self, interval_s: f64) -> CanvasBuilder {
        assert!(interval_s > 0.00001);
        self.frame_interval = Some(interval_s);
        self
    }

    /// Set the size of the logical canvas.
    pub fn set_size(mut self, width: i32, height: i32) -> CanvasBuilder {
        assert!(width > 0 && height > 0);
        self.dim = V2(width, height);
        self
    }

    /// Add an image into the canvas image atlas.
    pub fn add_image<P: Pixel<u8> + 'static, I: GenericImage<P>>(
        &mut self, offset: V2<i32>, image: I) -> Image {
        Image(self.builder.push(offset, image))
    }

    /// Start running the engine, return an event iteration.
    pub fn run(&mut self) -> Canvas {
        Canvas::new(
            self.dim,
            self.title.as_slice(),
            self.frame_interval,
            Atlas::new(&self.builder))
    }

    /// Load the default font into the texture atlas.
    fn init_font(&mut self) {
        let mut font_sheet = util::color_key(
            &image::load_from_memory(include_bytes!("../assets/font.png")).unwrap(),
            &Rgb::new(0x80u8, 0x80u8, 0x80u8));
        for i in 0..96 {
            let x = 8u32 * (i % 16u32);
            let y = 8u32 * (i / 16u32);
            let Image(idx) = self.add_image(V2(0, -8), SubImage::new(&mut font_sheet, x, y, 8, 8));
            assert!(idx - i as usize == FONT_IDX);
        }
    }

    /// Load a solid color element into the texture atlas.
    fn init_solid(&mut self) {
        let image: ImageBuffer<Vec<u8>, u8, Rgba<u8>> = ImageBuffer::from_fn(1, 1, Box::new(|&: _, _| Rgba([0xffu8, 0xffu8, 0xffu8, 0xffu8])));
        let Image(idx) = self.add_image(V2(0, 0), image);
        assert!(idx == SOLID_IDX);
    }
}

/// Interface to render to a live display.
pub struct Canvas {
    display: glium::Display,
    events: Vec<glutin::Event>,
    renderer: Renderer,

    atlas: Atlas,

    state: State,
    frame_interval: Option<f64>,
    last_render_time: f64,
    resolution: V2<i32>,
    window_resolution: V2<i32>,

    vertices: Vec<Vertex>,
    indices: Vec<u16>,

    /// Time in seconds it took to render the last frame.
    pub render_duration: f64,
}

#[derive(PartialEq)]
enum State {
    Normal,
    EndFrame,
}

impl Canvas {
    fn new(
        dim: V2<i32>,
        title: &str,
        frame_interval: Option<f64>,
        atlas: Atlas) -> Canvas {

        let display = glutin::WindowBuilder::new()
            .with_title(title.to_string())
            .with_dimensions(dim.0 as u32, dim.1 as u32)
            .build_glium().unwrap();

        let (w, h) = display.get_framebuffer_dimensions();

        let tex_image = image::imageops::flip_vertical(&atlas.image);
        let renderer = Renderer::new(&display, tex_image);

        Canvas {
            display: display,
            events: Vec::new(),
            renderer: renderer,

            atlas: atlas,

            state: State::Normal,
            frame_interval: frame_interval,
            last_render_time: time::precise_time_s(),
            resolution: dim,
            window_resolution: V2(w as i32, h as i32),

            vertices: Vec::new(),
            indices: Vec::new(),

            render_duration: 0.1f64,
        }
    }

    /// Clear the screen.
    pub fn clear(&mut self) {
        // TODO: use the color.
        self.vertices.clear();
        self.indices.clear();
    }

    /// Transform screen coordinates from the scaled-up window into
    /// pixel-perfect coordinates of the actual graphics.
    fn screen_to_pixel(&self, V2(sx, sy): V2<i32>) -> V2<i32> {
        let Rect(V2(rx, ry), V2(rw, rh)) = pixel_perfect(self.resolution, self.window_resolution);

        V2(((sx - rx) as f32 * self.resolution.0 as f32 / rw as f32) as i32,
           ((sy - ry) as f32 * self.resolution.1 as f32 / rh as f32) as i32)
    }

    #[inline(always)]
    fn window_to_device(&self, window_pos: V2<f32>, z: f32) -> [f32; 3] {
        let V2(w, h) = self.window_resolution;
        let Rect(V2(rx, ry), V2(rw, _)) = pixel_perfect(self.resolution, self.window_resolution);
        let zoom = (rw as f32) / (self.resolution.0 as f32);
        [-1.0 + (2.0 * (rx as f32 + window_pos.0 * zoom) / w as f32),
          1.0 - (2.0 * (ry as f32 + window_pos.1 * zoom) / h as f32),
         z]
    }

    /// Add a vertex to the geometry data of the current frame.
    pub fn push_vertex<C: Color, C2: Color>(&mut self, pos: V2<f32>, layer: f32, tex_coord: V2<f32>,
                                 color: &C, back_color: &C2) {
        let pos = self.window_to_device(pos, layer);

        self.vertices.push(Vertex {
            pos: pos,
            tex_coord: [tex_coord.0, tex_coord.1],
            color: color.to_rgba(),
            back_color: back_color.to_rgba(),
        });
    }

    /// Return the current vertex count, important for determining the indices
    /// for newly inserted vertices.
    pub fn num_vertices(&self) -> u16 { self.vertices.len() as u16 }

    /// Add a triangle defined by index values into the list of vertices
    /// inserted with push_vertex.
    pub fn push_triangle(&mut self, p0: u16, p1: u16, p2: u16) {
        self.indices.push(p0);
        self.indices.push(p1);
        self.indices.push(p2);
    }

    /// Return the image corresponding to a char in the built-in font.
    pub fn font_image(&self, c: char) -> Option<Image> {
        let idx = c as usize;
        // Hardcoded limiting of the font to printable ASCII.
        if idx >= 32 && idx < 128 {
            Some(Image((idx - 32) as usize + FONT_IDX))
        } else {
            None
        }
    }

    /// Return a texture coordinate to a #FFFFFFFF texel for solid color
    /// graphics.
    pub fn solid_tex_coord(&self) -> V2<f32> { self.atlas.items[SOLID_IDX].tex.0 }

    pub fn image_data<'a>(&'a self, Image(idx): Image) -> &'a AtlasItem {
        &self.atlas.items[idx]
    }
}

impl<'a> Iterator for Canvas {
    type Item=Event<'a>;

    fn next(&mut self) -> Option<Event<'a>> {
        // After a render event, control will return here on a new
        // iter call. Do post-render work here.
        if self.state == State::EndFrame {
            self.state = State::Normal;

            let mut target = self.display.draw();

            // Move out the accumulated geometry data.
            let vertices = mem::replace(&mut self.vertices, Vec::new());
            let indices = mem::replace(&mut self.indices, Vec::new());
            self.renderer.draw(&self.display, &mut target, vertices, indices);

            target.finish();
        }

        loop {
            self.events.push_all(self.display.poll_events().collect::<Vec<glutin::Event>>().as_slice());

            if !self.events.is_empty() {
                match self.events.remove(0) {
                    glutin::Event::ReceivedCharacter(ch) => {
                        return Some(Event::Char(ch));
                    }
                    glutin::Event::KeyboardInput(action, scan, _vko) => {
                        if (scan as usize) < scancode::MAP.len() {
                            if let Some(key) = scancode::MAP[scan as usize] {
                                return Some(if action == glutin::ElementState::Pressed {
                                    Event::KeyPressed(key)
                                }
                                else {
                                    Event::KeyReleased(key)
                                });
                            }
                        }
                    }
                    glutin::Event::MouseMoved((x, y)) => {
                        let pixel_pos = self.screen_to_pixel(V2(x, y));
                        return Some(Event::MouseMoved((pixel_pos.0, pixel_pos.1)));
                    }
                    glutin::Event::MouseWheel(x) => {
                        return Some(Event::MouseWheel(x));
                    }
                    glutin::Event::MouseInput(state, button) => {
                        let button = match button {
                            glutin::MouseButton::LeftMouseButton => MouseButton::Left,
                            glutin::MouseButton::RightMouseButton => MouseButton::Right,
                            glutin::MouseButton::MiddleMouseButton => MouseButton::Middle,
                            glutin::MouseButton::OtherMouseButton(x) => MouseButton::Other(x),
                        };
                        match state {
                            glutin::ElementState::Pressed => {
                                return Some(Event::MousePressed(button));
                            }
                            glutin::ElementState::Released => {
                                return Some(Event::MouseReleased(button));
                            }
                        }
                    }
                    glutin::Event::Focused(b) => {
                        return Some(Event::FocusChanged(b));
                    }
                    glutin::Event::Closed => {
                        return None;
                    }
                    _ => ()
                }
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

                let (w, h) = self.display.get_framebuffer_dimensions();
                self.window_resolution = V2(w as i32, h as i32);

                unsafe {
                    return Some(Event::Render(mem::transmute(self)))
                }
            }
        }
    }
}

/// Drawable images stored in the Canvas.
#[derive(Copy, Clone, PartialEq)]
pub struct Image(usize);

/// A pixel perfect centered and scaled rectangle of resolution dim in a
/// window of size area.
#[inline(always)]
fn pixel_perfect(canvas: V2<i32>, window: V2<i32>) -> Rect<i32> {
    let mut scale = (window.0 as f32 / canvas.0 as f32)
        .min(window.1 as f32 / canvas.1 as f32);

    if scale > 1.0 {
        // Snap to pixel scale if more than 1 window pixel per canvas pixel.
        scale = scale.floor();
    }

    let dim = V2((scale * canvas.0 as f32) as i32, (scale * canvas.1 as f32) as i32);
    let offset = (window - dim) / 2;
    Rect(offset, dim)
}
