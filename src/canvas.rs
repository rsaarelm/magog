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
use gfx::tex;
use gfx::{Device, DeviceHelper, ToSlice, CommandBuffer};
use gfx::{GlDevice};
use key;
use key::Key;
use atlas::{AtlasBuilder, Atlas};
use util;
use geom::{V2};
use event::Event;
use event;

pub struct Canvas {
    title: String,
    dim: V2<u32>,
    frame_interval: Option<f64>,
    builder: AtlasBuilder,
    font_glyphs: HashMap<char, Image>,
}

/// Toplevel graphics drawing and input reading context.
impl Canvas {
    pub fn new() -> Canvas {
        let mut ret = Canvas {
            title: "window".to_string(),
            dim: V2(640, 360),
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
    pub fn set_dim(mut self, dim: V2<u32>) -> Canvas {
        self.dim = dim;
        self
    }

    pub fn add_image<P: Pixel<u8>, I: GenericImage<P>>(
        &mut self, offset: V2<int>, image: I) -> Image {
        Image(self.builder.push(offset, image))
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
            &image::load_from_memory(include_bin!("../assets/font.png"), image::PNG).unwrap(),
            &Rgb::new(0x80u8, 0x80u8, 0x80u8));
        for i in range(0u32, 96u32) {
            let x = 8u32 * (i % 16u32);
            let y = 8u32 * (i / 16u32);
            let glyph = self.add_image(V2(0, 8), SubImage::new(&mut font_sheet, x, y, 8, 8));
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
    program: gfx::ProgramHandle,

    state: State,
    frame_interval: Option<f64>,
    last_render_time: f64,
    atlas: Atlas,
    atlas_tex: Texture,
    flatshade: gfx::TextureHandle,
    resolution: V2<u32>,
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

        let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        let (window, events) = glfw
            .create_window(dim.0, dim.1,
            title.as_slice(), glfw::Windowed)
            .expect("Failed to open window");
        window.make_current();
        glfw.set_error_callback(glfw::FAIL_ON_ERRORS);
        window.set_key_polling(true);
        window.set_char_polling(true);

        let device = gfx::GlDevice::new(|s| window.get_proc_address(s));
        let mut graphics = gfx::Graphics::new(device);
        //let frame = gfx::Frame::new(dim.0 as u16, dim.1 as u16);
        let (w, h) = window.get_framebuffer_size();
        let frame = gfx::Frame::new(w as u16, h as u16);
        let atlas_tex = Texture::from_rgba8(&atlas.image, &mut graphics.device);

        // Blank white texture so we can draw flatshaded stuff without
        // switching to non-texturing shader.
        let mut inf = tex::TextureInfo::new();
        inf.width = 1;
        inf.height = 1;
        inf.kind = tex::Texture2D;
        inf.format = tex::RGBA8;
        let flatshade = graphics.device.create_texture(inf).unwrap();
        graphics.device.update_texture(&flatshade, &inf.to_image_info(), &[0xffu8, 0xff, 0xff, 0xff]).unwrap();

        let program = graphics.device.link_program(VERTEX_SRC.clone(), FRAGMENT_SRC.clone()).unwrap();

        Context {
            glfw: glfw,
            window: window,
            events: events,
            graphics: graphics,
            frame: frame,
            program: program,

            state: Normal,
            frame_interval: frame_interval,
            last_render_time: time::precise_time_s(),
            atlas: atlas,
            atlas_tex: atlas_tex,
            flatshade: flatshade,
            resolution: dim,
        }
    }

    /// Clear the screen
    pub fn clear(&mut self, color: &Rgb) {
        self.graphics.clear(
            gfx::ClearData {
                color: color.to_array(),
                depth: 1.0,
                stencil: 0,
            }, gfx::Color | gfx::Depth, &self.frame);
    }

    pub fn draw_image(&mut self, offset: V2<int>, layer: f32, Image(idx): Image, color: &Rgb) {
        let scale = self.resolution.map(|x| 2.0 / (x as f32));

        let texcoords = self.atlas.texcoords[idx];
        let V2(u1, v1) = texcoords.mn();
        let V2(u2, v2) = texcoords.mx();

        let vertices = self.atlas.vertices[idx];
        let V2(x1, y1) = (vertices.mn() + offset.map(|x| x as f32)).mul(scale) - V2(1f32, 1f32);
        let V2(x2, y2) = (vertices.mx() + offset.map(|x| x as f32)).mul(scale) - V2(1f32, 1f32);

        let mesh = self.graphics.device.create_mesh([
            Vertex { pos: [x1, -y2, layer], tex_coord: [u1, v2] },
            Vertex { pos: [x1, -y1, layer], tex_coord: [u1, v1] },
            Vertex { pos: [x2, -y2, layer], tex_coord: [u2, v2] },

            Vertex { pos: [x2, -y2, layer], tex_coord: [u2, v2] },
            Vertex { pos: [x1, -y1, layer], tex_coord: [u1, v1] },
            Vertex { pos: [x2, -y1, layer], tex_coord: [u2, v1] },
        ]);

        let sampler_info = Some(self.graphics.device.create_sampler(
            tex::SamplerInfo::new(tex::Scale, tex::Clamp)));
        let params = ShaderParam {
            color: color.to_array(),
            s_texture: (self.atlas_tex.tex, sampler_info),
        };

        let slice = mesh.to_slice(gfx::TriangleList);
        let mut draw_state = gfx::DrawState::new()
            .depth(gfx::state::LessEqual, true);
        draw_state.primitive.front_face = gfx::state::Clockwise;
        let batch: gfx::batch::RefBatch<_ShaderParamLink, ShaderParam> = self.graphics.make_batch(
            &self.program, &mesh, slice, &draw_state).unwrap();
        self.graphics.draw(&batch, &params, &self.frame);
    }

    pub fn draw_line(&mut self, p1: V2<int>, p2: V2<int>, layer: f32, thickness: f32, color: &Rgb) {
        let scale = self.resolution.map(|x| 2.0 / (x as f32));
        let p1 = p1.map(|x| x as f32).mul(scale) - V2(1f32, 1f32);
        let p2 = p2.map(|x| x as f32).mul(scale) - V2(1f32, 1f32);

        let mesh = self.graphics.device.create_mesh([
            Vertex { pos: [p1.0, -p1.1, layer], tex_coord: [0.0, 0.0] },
            Vertex { pos: [p2.0, -p2.1, layer], tex_coord: [0.0, 0.0] },
        ]);
        // XXX: Copy-pasted code

        let sampler_info = Some(self.graphics.device.create_sampler(
            tex::SamplerInfo::new(tex::Scale, tex::Clamp)));
        let params = ShaderParam {
            color: color.to_array(),
            s_texture: (self.flatshade, sampler_info),
        };
        let slice = mesh.to_slice(gfx::Line);
        let mut draw_state = gfx::DrawState::new()
            .depth(gfx::state::LessEqual, true);
        draw_state.primitive.method = gfx::state::Line(thickness);
        let batch: gfx::batch::RefBatch<_ShaderParamLink, ShaderParam> = self.graphics.make_batch(
            &self.program, &mesh, slice, &draw_state).unwrap();
        self.graphics.draw(&batch, &params, &self.frame);
    }

    pub fn font_image(&self, c: char) -> Option<Image> {
        let idx = c as int;
        if idx >= 32 && idx < 128 {
            Some(Image((idx - 32) as uint))
        } else {
            None
        }
    }

    pub fn image_dim(&self, Image(idx): Image) -> V2<uint> {
        self.atlas.vertices[idx].1.map(|x| x as uint)
    }
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
                            return Some(event::Text(String::from_char(1, ch)));
                        }
                        glfw::KeyEvent(k, _scan, action, _mods) => {
                            match translate_glfw_key(k).map(|k| {
                                if action == glfw::Press || action == glfw::Repeat {
                                    event::KeyPressed(k)
                                }
                                else {
                                    event::KeyReleased(k)
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
                let (w, h) = self.window.get_framebuffer_size();
                self.frame = gfx::Frame::new(w as u16, h as u16);
                unsafe {
                    return Some(event::Render(mem::transmute(self)))
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

    attribute vec3 a_pos;
    attribute vec2 a_tex_coord;

    // TODO: Make color a uniform argument.
    varying vec2 v_tex_coord;
    varying vec4 v_color;

    void main() {
        v_tex_coord = a_tex_coord;
        v_color = color;
        gl_Position = vec4(a_pos, 1.0);
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
    pos: [f32, ..3],

    #[name = "a_tex_coord"]
    tex_coord: [f32, ..2],
}

impl Clone for Vertex {
    fn clone(&self) -> Vertex { *self }
}

struct Texture {
    tex: gfx::TextureHandle,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    fn from_rgba8<D: Device<C>, C: CommandBuffer>(
        img: &ImageBuf<Rgba<u8>>,
        d: &mut D) -> Texture {
        let (w, h) = img.dimensions();
        let mut info = tex::TextureInfo::new();
        info.width = w as u16;
        info.height = h as u16;
        info.kind = tex::Texture2D;
        info.format = tex::RGBA8;

        let tex = d.create_texture(info).unwrap();
        d.update_texture(&tex, &info.to_image_info(), img.pixelbuf()).unwrap();

        Texture {
            tex: tex,
            width: w,
            height: h,
        }
    }
}

fn translate_glfw_key(k: glfw::Key) -> Option<Key> {
    match k {
        glfw::KeySpace => Some(key::KeySpace),
        glfw::KeyApostrophe => Some(key::KeyApostrophe),
        glfw::KeyComma => Some(key::KeyComma),
        glfw::KeyMinus => Some(key::KeyMinus),
        glfw::KeyPeriod => Some(key::KeyPeriod),
        glfw::KeySlash => Some(key::KeySlash),
        glfw::Key0 => Some(key::Key0),
        glfw::Key1 => Some(key::Key1),
        glfw::Key2 => Some(key::Key2),
        glfw::Key3 => Some(key::Key3),
        glfw::Key4 => Some(key::Key4),
        glfw::Key5 => Some(key::Key5),
        glfw::Key6 => Some(key::Key6),
        glfw::Key7 => Some(key::Key7),
        glfw::Key8 => Some(key::Key8),
        glfw::Key9 => Some(key::Key9),
        glfw::KeySemicolon => Some(key::KeySemicolon),
        glfw::KeyEqual => Some(key::KeyEquals),
        glfw::KeyA => Some(key::KeyA),
        glfw::KeyB => Some(key::KeyB),
        glfw::KeyC => Some(key::KeyC),
        glfw::KeyD => Some(key::KeyD),
        glfw::KeyE => Some(key::KeyE),
        glfw::KeyF => Some(key::KeyF),
        glfw::KeyG => Some(key::KeyG),
        glfw::KeyH => Some(key::KeyH),
        glfw::KeyI => Some(key::KeyI),
        glfw::KeyJ => Some(key::KeyJ),
        glfw::KeyK => Some(key::KeyK),
        glfw::KeyL => Some(key::KeyL),
        glfw::KeyM => Some(key::KeyM),
        glfw::KeyN => Some(key::KeyN),
        glfw::KeyO => Some(key::KeyO),
        glfw::KeyP => Some(key::KeyP),
        glfw::KeyQ => Some(key::KeyQ),
        glfw::KeyR => Some(key::KeyR),
        glfw::KeyS => Some(key::KeyS),
        glfw::KeyT => Some(key::KeyT),
        glfw::KeyU => Some(key::KeyU),
        glfw::KeyV => Some(key::KeyV),
        glfw::KeyW => Some(key::KeyW),
        glfw::KeyX => Some(key::KeyX),
        glfw::KeyY => Some(key::KeyY),
        glfw::KeyZ => Some(key::KeyZ),
        glfw::KeyLeftBracket => Some(key::KeyLeftBracket),
        glfw::KeyBackslash => Some(key::KeyBackslash),
        glfw::KeyRightBracket => Some(key::KeyRightBracket),
        glfw::KeyGraveAccent => Some(key::KeyGrave),
        glfw::KeyEscape => Some(key::KeyEscape),
        glfw::KeyEnter => Some(key::KeyEnter),
        glfw::KeyTab => Some(key::KeyTab),
        glfw::KeyBackspace => Some(key::KeyBackspace),
        glfw::KeyInsert => Some(key::KeyInsert),
        glfw::KeyDelete => Some(key::KeyDelete),
        glfw::KeyRight => Some(key::KeyRight),
        glfw::KeyLeft => Some(key::KeyLeft),
        glfw::KeyDown => Some(key::KeyDown),
        glfw::KeyUp => Some(key::KeyUp),
        glfw::KeyPageUp => Some(key::KeyPageUp),
        glfw::KeyPageDown => Some(key::KeyPageDown),
        glfw::KeyHome => Some(key::KeyHome),
        glfw::KeyEnd => Some(key::KeyEnd),
        glfw::KeyCapsLock => Some(key::KeyCapsLock),
        glfw::KeyScrollLock => Some(key::KeyScrollLock),
        glfw::KeyNumLock => Some(key::KeyNumLock),
        glfw::KeyPrintScreen => Some(key::KeyPrintScreen),
        glfw::KeyPause => Some(key::KeyPause),
        glfw::KeyF1 => Some(key::KeyF1),
        glfw::KeyF2 => Some(key::KeyF2),
        glfw::KeyF3 => Some(key::KeyF3),
        glfw::KeyF4 => Some(key::KeyF4),
        glfw::KeyF5 => Some(key::KeyF5),
        glfw::KeyF6 => Some(key::KeyF6),
        glfw::KeyF7 => Some(key::KeyF7),
        glfw::KeyF8 => Some(key::KeyF8),
        glfw::KeyF9 => Some(key::KeyF9),
        glfw::KeyF10 => Some(key::KeyF10),
        glfw::KeyF11 => Some(key::KeyF11),
        glfw::KeyF12 => Some(key::KeyF12),
        glfw::KeyKp0 => Some(key::KeyPad0),
        glfw::KeyKp1 => Some(key::KeyPad1),
        glfw::KeyKp2 => Some(key::KeyPad2),
        glfw::KeyKp3 => Some(key::KeyPad3),
        glfw::KeyKp4 => Some(key::KeyPad4),
        glfw::KeyKp5 => Some(key::KeyPad5),
        glfw::KeyKp6 => Some(key::KeyPad6),
        glfw::KeyKp7 => Some(key::KeyPad7),
        glfw::KeyKp8 => Some(key::KeyPad8),
        glfw::KeyKp9 => Some(key::KeyPad9),
        glfw::KeyKpDecimal => Some(key::KeyPadDecimal),
        glfw::KeyKpDivide => Some(key::KeyPadDivide),
        glfw::KeyKpMultiply => Some(key::KeyPadMultiply),
        glfw::KeyKpSubtract => Some(key::KeyPadMinus),
        glfw::KeyKpAdd => Some(key::KeyPadPlus),
        glfw::KeyKpEnter => Some(key::KeyPadEnter),
        glfw::KeyKpEqual => Some(key::KeyPadEquals),
        glfw::KeyLeftShift => Some(key::KeyLeftShift),
        glfw::KeyLeftControl => Some(key::KeyLeftControl),
        glfw::KeyLeftAlt => Some(key::KeyLeftAlt),
        glfw::KeyLeftSuper => Some(key::KeyLeftSuper),
        glfw::KeyRightShift => Some(key::KeyRightShift),
        glfw::KeyRightControl => Some(key::KeyRightControl),
        glfw::KeyRightAlt => Some(key::KeyRightAlt),
        glfw::KeyRightSuper => Some(key::KeyRightSuper),
        _ => None
    }
}

#[deriving(Clone, PartialEq, Eq, Show)]
pub struct Rgb { pub r: u8, pub g: u8, pub b: u8 }

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Rgb {
        Rgb { r: r, g: g, b: b }
    }

    pub fn to_array(&self) -> [f32, ..4] {
        [self.r as f32 / 255.0,
         self.g as f32 / 255.0,
         self.b as f32 / 255.0,
         1.0]
    }
}
