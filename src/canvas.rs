use std::collections::hashmap::HashMap;
use std::str;
use time;
use std::mem;
use sync::comm::Receiver;
use image::{GenericImage, SubImage};
use image::{Pixel, ImageBuf, Rgba};
use image;
use glfw;
use glfw::Context as _Context;
use gfx;
use gfx::{DeviceHelper, ToSlice};
use util;
use key;

static FONT_DATA: &'static [u8] = include_bin!("../assets/font.png");

pub struct Canvas {
    title: String,
    dim: [u32, ..2],
    frame_interval: Option<f64>,
    image_collector: ImageCollector,
    font_glyphs: HashMap<char, Image>,
}

/// Toplevel graphics drawing and input reading context.
impl Canvas {
    pub fn new() -> Canvas {
        let mut ret = Canvas {
            title: "window".to_string(),
            dim: [640, 360],
            frame_interval: None,
            image_collector: ImageCollector::new(),
            font_glyphs: HashMap::new(),
        };
        ret.init_font();
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
    pub fn set_dim(mut self, dim: [u32, ..2]) -> Canvas {
        self.dim = dim;
        self
    }

    pub fn add_image<P: Pixel<u8>, I: GenericImage<P>>(
        &mut self, image: I) -> Image {
        Image(self.image_collector.push(image))
    }

    /// Start running the engine, return an event iteration.
    pub fn run(&mut self) -> Context {
        Context::new(
            self.dim,
            self.title.as_slice(),
            Some(1.0 / 30.0),
            self.image_collector.build_atlas())
    }

    fn init_font(&mut self) {
        let mut font_sheet = image::load_from_memory(FONT_DATA, image::PNG).unwrap();
        for i in range(0u32, 96u32) {
            let x = 8u32 * (i % 16u32);
            let y = 8u32 * (i / 16u32);
            let glyph = self.add_image(SubImage::new(&mut font_sheet, x, y, 8, 8));
            self.font_glyphs.insert((i + 32) as u8 as char, glyph);
        }
    }
}

pub struct Context {
    glfw: glfw::Glfw,
    window: glfw::Window,
    events: Receiver<(f64, glfw::WindowEvent)>,
    graphics: gfx::Graphics<gfx::GlDevice, gfx::GlCommandBuffer>,
    frame: gfx::Frame,

    state: State,
    frame_interval: Option<f64>,
    last_render_time: f64,
    atlas: Atlas,
}

#[deriving(PartialEq)]
enum State {
    Normal,
    EndFrame,
}

impl Context {
    fn new(
        dim: [u32, ..2],
        title: &str,
        frame_interval: Option<f64>,
        atlas: Atlas) -> Context {

        let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        let (window, events) = glfw
            .create_window(dim[0], dim[1],
            title.as_slice(), glfw::Windowed)
            .expect("Failed to open window");
        window.make_current();
        glfw.set_error_callback(glfw::FAIL_ON_ERRORS);
        window.set_key_polling(true);
        window.set_char_polling(true);

        let device = gfx::GlDevice::new(|s| window.get_proc_address(s));
        let graphics = gfx::Graphics::new(device);
        let frame = gfx::Frame::new(dim[0] as u16, dim[1] as u16);

        Context {
            glfw: glfw,
            window: window,
            events: events,
            graphics: graphics,
            frame: frame,
            state: Normal,
            frame_interval: frame_interval,
            last_render_time: time::precise_time_s(),
            atlas: atlas,
        }
    }

    /// Clear the screen
    pub fn clear(&mut self, color: [f32, ..4]) {
        self.graphics.clear(
            gfx::ClearData {
                color: color,
                depth: 1.0,
                stencil: 0,
            }, gfx::Color, &self.frame);
    }

    /// Mess with drawy stuff
    pub fn draw_test(&mut self) {
        let mesh = self.graphics.device.create_mesh(vec![
            Vertex { pos: [0.0, 0.0], tex_coord: [0.0, 0.0] },
            Vertex { pos: [1.0, 0.0], tex_coord: [1.0, 0.0] },
            Vertex { pos: [0.0, 1.0], tex_coord: [0.0, 1.0] },
        ]);
        let slice = mesh.to_slice(gfx::TriangleList);
        let program = self.graphics.device.link_program(
            VERTEX_SRC.clone(), FRAGMENT_SRC.clone()).unwrap();
        let batch: gfx::batch::RefBatch<(), ()> = self.graphics.make_batch(
            &program, &mesh, slice, &gfx::DrawState::new()).unwrap();
        self.graphics.draw(&batch, &(), &self.frame);
    }

    pub fn draw_image(&mut self, offset: [int, ..2], image: Image) {
        unimplemented!();
    }
}

pub enum Event<'a> {
    /// Time to render the screen. Call your own render code on the Context
    /// value when you get this.
    Render(&'a mut Context),
    /// Some printable text entered. Data is whole strings to account for
    /// exotic input devices.
    Text(String),
    KeyPressed(key::Key),
    KeyReleased(key::Key),
}

impl<'a> Iterator<Event<'a>> for Context {
    fn next(&mut self) -> Option<Event<'a>> {
        // After a render event, control will return here on a new
        // iter call. Do post-render work here.
        if self.state == EndFrame {
            self.state = Normal;
            self.graphics.end_frame();
            self.window.swap_buffers();
        }

        loop {
            if self.window.should_close() {
                return None;
            }

            self.glfw.poll_events();

            match self.events.try_recv() {
                Ok((_, event)) => {
                    match event {
                        glfw::CharEvent(ch) => {
                            return Some(Text(str::from_char(ch)));
                        }
                        glfw::KeyEvent(k, _scan, action, _mods) => {
                            match key::translate_glfw_key(k).map(|k| {
                                if action == glfw::Press || action == glfw::Repeat {
                                    KeyPressed(k)
                                }
                                else {
                                    KeyReleased(k)
                                }
                            }) {
                                Some(e) => { return Some(e); }
                                _ => ()
                            }
                        }
                        // TODO Mouse events.
                        _ => ()
                    }
                }
                _ => ()
            }

            let t = time::precise_time_s();
            if self.frame_interval.map_or(true,
                |x| t - self.last_render_time >= x) {
                self.last_render_time = t;

                // Time to render, must return a handle to self.
                // XXX: Need unsafe hackery to get around lifetimes check.
                self.state = EndFrame;
                unsafe {
                    return Some(Render(mem::transmute(self)))
                }
            }
        }
    }
}

/// Drawable images stored in the Canvas.
#[deriving(Clone, PartialEq)]
pub struct Image(uint);

struct ImageCollector {
    images: Vec<ImageBuf<Rgba<u8>>>,
    offsets: Vec<[u32, ..2]>,
}

impl ImageCollector {
    fn new() -> ImageCollector {
        ImageCollector {
            images: vec![],
            offsets: vec![],
        }
    }

    fn push<P: Pixel<u8>, I: GenericImage<P>>(
        &mut self, mut image: I) -> uint {
        let (pos, dim) = util::crop_alpha(&image);
        let cropped = SubImage::new(&mut image, pos[0], pos[1], dim[0], dim[1]);

        let (w, h) = cropped.dimensions();
        let img = ImageBuf::from_pixels(
            cropped.pixels().map::<Rgba<u8>>(
                |(_x, _y, p)| p.to_rgba())
            .collect(),
            w, h);
        self.images.push(img);
        self.offsets.push(pos);
        self.images.len()
    }

    fn build_atlas(&mut self) -> Atlas {
        Atlas::new(&self.images, &self.offsets)
    }

    fn rendering_offset(&self, idx: uint) -> [u32, ..2] { self.offsets[idx] }
}

struct Atlas {
    image: ImageBuf<Rgba<u8>>,
    bounds: Vec<([u32, ..2], [u32, ..2])>,
    offsets: Vec<[u32, ..2]>,
}

impl Atlas {
    fn new(images: &Vec<ImageBuf<Rgba<u8>>>, offsets: &Vec<[u32, ..2]>) -> Atlas {
        let (image, bounds) = util::build_atlas(images);

        // TODO: Replace with offsets.clone() when Rust supports fixed size array cloning.
        let mut offsets_clone = vec![]; for i in offsets.iter() { offsets_clone.push(*i); }

        Atlas {
            image: image,
            bounds: bounds,
            offsets: offsets_clone,
        }
    }
}

#[vertex_format]
struct Vertex {
    #[name = "a_pos"]
    pos: [f32, ..2],

    #[name = "a_tex_coord"]
    tex_coord: [f32, ..2],
}

static VERTEX_SRC: gfx::ShaderSource = shaders! {
GLSL_120: b"
    #version 120

    attribute vec2 a_pos;
    attribute vec2 a_tex_coord;
    // TODO: Make color a uniform argument.
    varying vec4 v_color;

    void main() {
        v_color = vec4(1.0, 0.0, 0.0, 1.0);
        gl_Position = vec4(a_pos, 0.0, 1.0);
    }
"
};

static FRAGMENT_SRC: gfx::ShaderSource = shaders! {
GLSL_120: b"
    #version 120

    varying vec4 v_color;

    void main() {
        gl_FragColor = v_color;
    }
"
};
