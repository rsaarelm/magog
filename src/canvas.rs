use std::collections::hashmap::HashMap;
use time;
use std::mem;
use sync::comm::Receiver;
use image::{GenericImage, SubImage, Pixel};
use image::{ImageBuf, Rgba};
use image;
use glfw;
use glfw::Context as _Context;
use gfx;
use gfx::{Device, DeviceHelper, ToSlice, CommandBuffer};
use gfx::{GlDevice};
use key;
use atlas::{AtlasBuilder, Atlas};
use util;
use color;
use color::{Rgb};

static FONT_DATA: &'static [u8] = include_bin!("../assets/font.png");

pub struct Canvas {
    title: String,
    dim: [u32, ..2],
    frame_interval: Option<f64>,
    builder: AtlasBuilder,
    font_glyphs: HashMap<char, Image>,
}

/// Toplevel graphics drawing and input reading context.
impl Canvas {
    pub fn new() -> Canvas {
        let mut ret = Canvas {
            title: "window".to_string(),
            dim: [640, 360],
            frame_interval: None,
            builder: AtlasBuilder::new(),
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
        Image(self.builder.push(image))
    }

    /// Start running the engine, return an event iteration.
    pub fn run(&mut self) -> Context {
        Context::new(
            self.dim,
            self.title.as_slice(),
            Some(1.0 / 30.0),
            Atlas::new(&self.builder))
    }

    fn init_font(&mut self) {
        let mut font_sheet = util::color_key(
            &image::load_from_memory(FONT_DATA, image::PNG).unwrap(),
            0x80u8, 0x80u8, 0x80u8);
        for i in range(0u32, 96u32) {
            let x = 8u32 * (i % 16u32);
            let y = 8u32 * (i / 16u32);
            let glyph = self.add_image(SubImage::new(&mut font_sheet, x, y, 8, 8));
            self.font_glyphs.insert((i + 32) as u8 as char, glyph);
        }
    }
}

/// Interface to render to a live display.
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
    atlas_tex: Texture,
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
        let mut graphics = gfx::Graphics::new(device);
        let frame = gfx::Frame::new(dim[0] as u16, dim[1] as u16);
        let atlas_tex = Texture::from_rgba8(&atlas.image, &mut graphics.device);

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
            atlas_tex: atlas_tex,
        }
    }

    /// Clear the screen
    pub fn clear(&mut self, color: &Rgb) {
        self.graphics.clear(
            gfx::ClearData {
                color: color.to_array(),
                depth: 1.0,
                stencil: 0,
            }, gfx::Color, &self.frame);
    }

    /// Mess with drawy stuff
    pub fn draw_test(&mut self) {
        let mesh = self.graphics.device.create_mesh([
            Vertex { pos: [0.0, 1.0], tex_coord: [0.0, 0.0] },
            Vertex { pos: [1.0, 1.0], tex_coord: [1.0, 0.0] },
            Vertex { pos: [0.0, 0.0], tex_coord: [0.0, 1.0] },

            Vertex { pos: [1.0, 1.0], tex_coord: [1.0, 0.0] },
            Vertex { pos: [0.0, 0.0], tex_coord: [0.0, 1.0] },
            Vertex { pos: [1.0, 0.0], tex_coord: [1.0, 1.0] },
        ]);

        let sampler_info = None; // TODO
        let params = ShaderParam {
            color: [1.0, 0.0, 1.0, 0.0],
            s_texture: (self.atlas_tex.tex, sampler_info),
        };

        let slice = mesh.to_slice(gfx::TriangleList);
        let program = self.graphics.device.link_program(
            VERTEX_SRC.clone(), FRAGMENT_SRC.clone()).unwrap();
        let batch: gfx::batch::RefBatch<_ShaderParamLink, ShaderParam> = self.graphics.make_batch(
            &program, &mesh, slice, &gfx::DrawState::new()).unwrap();
        self.graphics.draw(&batch, &params, &self.frame);
    }

    pub fn draw_image(&mut self, offset: (int, int), image: Image) {
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
                            return Some(Text(String::from_char(1, ch)));
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

static VERTEX_SRC: gfx::ShaderSource = shaders! {
GLSL_120: b"
    #version 120

    uniform vec4 color;

    attribute vec2 a_pos;
    attribute vec2 a_tex_coord;

    // TODO: Make color a uniform argument.
    varying vec2 v_tex_coord;
    varying vec4 v_color;

    void main() {
        v_tex_coord = a_tex_coord;
        v_color = color;
        gl_Position = vec4(a_pos, 0.0, 1.0);
    }
"
};

static FRAGMENT_SRC: gfx::ShaderSource = shaders! {
GLSL_120: b"
    #version 120

    uniform sampler2D s_texture;

    varying vec2 v_tex_coord;
    varying vec4 v_color;

    void main() {
        vec4 tex_color = texture2D(s_texture, v_tex_coord);
        if (tex_color.a == 0.0) discard;
        gl_FragColor = v_color * tex_color;
    }
"
};

#[shader_param(Program)]
pub struct ShaderParam {
    pub color: [f32, ..4],
    pub s_texture: gfx::shade::TextureParam,
}

#[vertex_format]
struct Vertex {
    #[name = "a_pos"]
    pos: [f32, ..2],

    #[name = "a_tex_coord"]
    tex_coord: [f32, ..2],
}

impl Clone for Vertex {
    fn clone(&self) -> Vertex { *self }
}

struct Texture {
    tex: gfx::TextureHandle,
    width: u32,
    height: u32,
}

impl Texture {
    fn from_rgba8<D: Device<C>, C: CommandBuffer>(
        img: &ImageBuf<Rgba<u8>>,
        d: &mut D) -> Texture {
        let (w, h) = img.dimensions();
        let mut info = gfx::tex::TextureInfo::new();
        info.width = w as u16;
        info.height = h as u16;
        info.kind = gfx::tex::Texture2D;
        info.format = gfx::tex::RGBA8;

        let tex = d.create_texture(info).unwrap();
        d.update_texture(&tex, &info.to_image_info(), img.pixelbuf()).unwrap();

        Texture {
            tex: tex,
            width: w,
            height: h,
        }
    }
}
