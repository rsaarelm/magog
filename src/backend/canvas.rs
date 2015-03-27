use time;
use std::mem;
use std::time::duration::Duration;
use std::thread;
use std::default::Default;
use image::{GenericImage, SubImage, Pixel};
use image::{ImageBuffer, Rgba};
use image;
use glutin;
use glium::{self, DisplayBuild};
use ::{AtlasBuilder, Atlas, AtlasItem, V2, Rgb, Color};
use super::event::{Event, MouseButton};
use super::renderer::{Renderer, Vertex};
use super::scancode;
use super::{WidgetId, CanvasMagnify};

/// Width of the full font cell. Actual variable-width letters occupy some
/// portion of the left side of the cell.
pub static FONT_W: u32 = 8;
/// Height of the font.
pub static FONT_H: u32 = 8;

/// The first font image in the atlas image set.
static FONT_IDX: usize = 0;

/// Image index of the solid color texture block.
static SOLID_IDX: usize = 96;

pub struct CanvasBuilder {
    title: String,
    size: V2<u32>,
    frame_interval: Option<f64>,
    fullscreen: bool,
    magnify: CanvasMagnify,
    atlas_builder: AtlasBuilder,
}

/// Toplevel graphics drawing and input reading context.
impl CanvasBuilder {
    pub fn new() -> CanvasBuilder {
        let mut ret = CanvasBuilder {
            title: "".to_string(),
            size: V2(640, 360),
            frame_interval: None,
            fullscreen: false,
            magnify: CanvasMagnify::PixelPerfect,
            atlas_builder: AtlasBuilder::new(),
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
    pub fn set_size(mut self, width: u32, height: u32) -> CanvasBuilder {
        self.size = V2(width, height);
        self
    }

    /// Set the canvas to start in fullscreen mode.
    /// XXX: Doesn't work right as of 2015-03-26, make sure this is fixed
    /// before using.
    pub fn _set_fullscreen(mut self) -> CanvasBuilder {
        self.fullscreen = true;
        self
    }

    pub fn set_magnify(mut self, magnify: CanvasMagnify) -> CanvasBuilder {
        self.magnify = magnify;
        self
    }

    /// Add an image into the canvas image atlas.
    pub fn add_image<P: Pixel<Subpixel=u8> + 'static, I: GenericImage<Pixel=P>>(
        &mut self, offset: V2<i32>, image: &I) -> Image {
        Image(self.atlas_builder.push(offset, image))
    }

    /// Start running the engine, return an event iteration.
    pub fn run(self) -> Canvas {
        Canvas::new(self)
    }

    /// Load the default font into the texture atlas.
    fn init_font(&mut self) {
        let mut font_sheet = ::color_key(
            &image::load_from_memory(include_bytes!("../assets/font.png")).unwrap(),
            &Rgb::new(0x80u8, 0x80u8, 0x80u8));
        for i in 0u32..96 {
            let x = 8u32 * (i % 16u32);
            let y = 8u32 * (i / 16u32);
            let Image(idx) = self.add_image(
                V2(0, -8),
                &SubImage::new(&mut font_sheet, x, y, 8, 8));
            assert!(idx - i as usize == FONT_IDX);
        }
    }

    /// Load a solid color element into the texture atlas.
    fn init_solid(&mut self) {
        let image: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_fn(1, 1, |_, _| Rgba([0xffu8, 0xffu8, 0xffu8, 0xffu8]));
        let Image(idx) = self.add_image(V2(0, 0), &image);
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
    size: V2<u32>,
    window_resolution: V2<i32>,

    vertices: Vec<Vertex>,
    indices: Vec<u16>,

    /// Time in seconds it took to render the last frame.
    pub render_duration: f64,

    pub mouse_pos: V2<f32>,
    pub mouse_pressed: bool,
    /// Imgui widget currently under mouse cursor.
    pub hot_widget: Option<WidgetId>,
    /// Imgui widget currently being interacted with.
    pub active_widget: Option<WidgetId>,
    /// Previous imgui widget.
    pub last_widget: Option<WidgetId>,
}

#[derive(PartialEq)]
enum State {
    Normal,
    EndFrame,
}

impl Canvas {
    fn new(builder: CanvasBuilder) -> Canvas {
        let size = builder.size;
        let title = &builder.title[..];
        let frame_interval = builder.frame_interval;
        let atlas = Atlas::new(&builder.atlas_builder);

        let mut glutin = glutin::WindowBuilder::new()
            .with_title(title.to_string());

        if builder.fullscreen {
            glutin = glutin.with_fullscreen(glutin::get_primary_monitor());
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

        let tex_image = image::imageops::flip_vertical(&atlas.image);
        let renderer = Renderer::new(size, &display, tex_image, builder.magnify);

        Canvas {
            display: display,
            events: Vec::new(),
            renderer: renderer,

            atlas: atlas,

            state: State::Normal,
            frame_interval: frame_interval,
            last_render_time: time::precise_time_s(),
            size: size,
            window_resolution: V2(w as i32, h as i32),

            vertices: Vec::new(),
            indices: Vec::new(),

            render_duration: 0.1f64,

            mouse_pos: Default::default(),
            mouse_pressed: false,
            hot_widget: None,
            active_widget: None,
            last_widget: None,
        }
    }

    /// Clear the screen.
    pub fn clear(&mut self) {
        // TODO: use the color.
        self.vertices.clear();
        self.indices.clear();
    }

    #[inline(always)]
    fn canvas_to_device(&self, pos: V2<f32>, z: f32) -> [f32; 3] {
        [-1.0 + (2.0 * (pos.0) / self.size.0 as f32),
          1.0 - (2.0 * (pos.1) / self.size.1 as f32),
         z]
    }

    /// Add a vertex to the geometry data of the current frame.
    pub fn push_vertex<C: Color, C2: Color>(&mut self, pos: V2<f32>, layer: f32, tex_coord: V2<f32>,
                                 color: &C, back_color: &C2) {
        let pos = self.canvas_to_device(pos, layer);

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
            Some(Image(idx - 32 + FONT_IDX))
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

    /// Return a screenshot image of the last frame rendered.
    pub fn screenshot(&self) -> ImageBuffer<image::Rgb<u8>, Vec<u8>> {
        self.renderer.canvas_pixels()
    }

    fn imgui_prepare(&mut self) {
        // Initial setup for imgui.
        self.hot_widget = None;
    }

    fn imgui_finish(&mut self) {
        if !self.mouse_pressed {
            self.active_widget = None;
        } else {
            // Setup a dummy widget so that dragging a mouse onto a widget
            // with the button held down won't activate that widget.
            if self.active_widget.is_none() {
                self.active_widget = Some(WidgetId::dummy());
            }
        }
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

            self.imgui_finish();
        }

        let mut app_focused = true;
        loop {
            self.events.push_all(&self.display.poll_events().collect::<Vec<glutin::Event>>()[..]);

            if !self.events.is_empty() {
                app_focused = true;
                match self.events.remove(0) {
                    glutin::Event::Focused(false) => { app_focused = false; }
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
                        let pixel_pos = self.renderer.screen_to_canvas(V2(x, y));
                        self.mouse_pos = pixel_pos.map(|x| x as f32);
                        return Some(Event::MouseMoved((pixel_pos.0, pixel_pos.1)));
                    }
                    glutin::Event::MouseWheel(x) => {
                        return Some(Event::MouseWheel(x));
                    }
                    glutin::Event::MouseInput(state, button) => {
                        let button = match button {
                            glutin::MouseButton::Left => MouseButton::Left,
                            glutin::MouseButton::Right => MouseButton::Right,
                            glutin::MouseButton::Middle => MouseButton::Middle,
                            glutin::MouseButton::Other(x) => MouseButton::Other(x),
                        };
                        self.mouse_pressed =
                            button == MouseButton::Left &&
                            state == glutin::ElementState::Pressed;
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
            if app_focused && self.frame_interval.map_or(true,
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

                self.imgui_prepare();

                // Return the render callback.
                unsafe {
                    return Some(Event::Render(mem::transmute(self)))
                }
            } else {
                // Go to sleep if there's time left.
                if let Some(mut remaining_s) = self.frame_interval {
                    remaining_s -= t - self.last_render_time;
                    if remaining_s > 0.0 {
                        thread::sleep(
                            Duration::nanoseconds((remaining_s * 1e9) as i64));
                    }
                }
            }
        }
    }
}

/// Drawable images stored in the Canvas.
#[derive(Copy, Clone, PartialEq)]
pub struct Image(usize);
