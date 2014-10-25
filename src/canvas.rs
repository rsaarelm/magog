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
use gfx::Mesh;
use atlas::{AtlasBuilder, Atlas};
use util;
use geom::{V2, Rect};
use event::Event;
use event;
use rgb::Rgb;
use glfw_key;

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

    /// Load the default font into the texture atlas.
    fn init_font(&mut self) {
        let mut font_sheet = util::color_key(
            &image::load_from_memory(include_bin!("../assets/font.png"), image::PNG).unwrap(),
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
        let image: ImageBuf<Rgba<u8>> = ImageBuf::from_fn(1, 1, |_, _| Rgba(0xffu8, 0xffu8, 0xffu8, 0xffu8));
        let Image(idx) = self.add_image(V2(0, 0), image);
        assert!(idx == SOLID_IDX);
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
    atlas_tex: Texture,
    meshes: Vec<Mesh>,
    image_dims: Vec<V2<uint>>,
    line_mesh: Mesh,
    resolution: V2<u32>,

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

        let mut meshes = vec![];
        let mut dims = vec![];

        for i in range(0, atlas.vertices.len()) {
            let V2(x1, y1) = atlas.vertices[i].mn();
            let V2(x2, y2) = atlas.vertices[i].mx();
            let V2(u1, v1) = atlas.texcoords[i].mn();
            let V2(u2, v2) = atlas.texcoords[i].mx();
            let mesh = graphics.device.create_mesh([
                Vertex { pos: [x1, y2, 0.0], tex_coord: [u1, v2] },
                Vertex { pos: [x1, y1, 0.0], tex_coord: [u1, v1] },
                Vertex { pos: [x2, y2, 0.0], tex_coord: [u2, v2] },

                Vertex { pos: [x2, y2, 0.0], tex_coord: [u2, v2] },
                Vertex { pos: [x1, y1, 0.0], tex_coord: [u1, v1] },
                Vertex { pos: [x2, y1, 0.0], tex_coord: [u2, v1] },
            ]);

            meshes.push(mesh);
            dims.push(atlas.vertices[i].1.map(|x| x as uint));
        }

        // The center of the solid image should be good as texture coordinates
        // for flatshade objects.
        let solid_uv = (atlas.texcoords[SOLID_IDX].mn() + atlas.texcoords[SOLID_IDX].mx()) * 0.5;

        let line_mesh = graphics.device.create_mesh([
                Vertex { pos: [0.0, 0.0, 0.0], tex_coord: [solid_uv.0, solid_uv.1] },
                Vertex { pos: [1.0, 1.0, 0.0], tex_coord: [solid_uv.0, solid_uv.1] },
        ]);

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
            atlas_tex: atlas_tex,
            meshes: meshes,
            image_dims: dims,
            line_mesh: line_mesh,
            resolution: dim,

            render_duration: 0.1f64,
        }
    }

    /// Clear the screen
    pub fn clear(&mut self, color: &Rgb) {
        self.graphics.clear(
            gfx::ClearData {
                color: color.to_array(),
                depth: 1.0,
                stencil: 0,
            }, gfx::COLOR | gfx::DEPTH, &self.frame);
    }

    /// Return the pixel-perfect scaled and centered canvas draw rectangle
    /// within the current window, in pixel coordinates. Return (rect_x,
    /// rect_y, rect_w, rect_h, window_w, window_h).
    fn canvas_rect(&self) -> (u32, u32, u32, u32, u32, u32) {
        let (w, h) = self.window.get_framebuffer_size();
        let (w, h) = (w as u32, h as u32);
        let Rect(V2(rx, ry), V2(rw, rh)) = pixel_perfect(self.resolution, V2(w, h));
        (rx, ry, rw, rh, w, h)
    }

    fn draw_state(&self) -> gfx::DrawState {
        let mut ret = gfx::DrawState::new()
            .depth(gfx::state::LessEqual, true);

        let (x, y, w, h, _, _) = self.canvas_rect();
        ret.scissor = Some(gfx::Rect {
            x: x as u16, y: y as u16, w: w as u16, h: h as u16
        });
        ret.primitive.front_face = gfx::state::Clockwise;

        ret
    }

    fn transform(&self, offset: V2<int>, scale: V2<int>, layer: f32) -> Transform {
        let (x, y, w, _, ww, wh) = self.canvas_rect();
        let zoom = (w as f32) / (self.resolution.0 as f32);
        let offset = offset + V2((x as f32 / zoom) as int, (y as f32 / zoom) as int);

        //let mut screen_scale = self.resolution.map(|x| 2.0 / (x as f32));
        let mut screen_scale = V2(2.0 * zoom / (ww as f32), 2.0 * zoom / (wh as f32));
        screen_scale.1 = -screen_scale.1;
        transform(
            scale.map(|x| x as f32).mul(screen_scale),
            offset.map(|x| x as f32).mul(screen_scale) - V2(1.0, -1.0),
            layer)
    }

    pub fn draw_image(&mut self, offset: V2<int>, layer: f32, Image(idx): Image, color: &Rgb) {
        let sampler_info = Some(self.graphics.device.create_sampler(
            tex::SamplerInfo::new(tex::Scale, tex::Clamp)));
        let params = ShaderParam {
            u_color: color.to_array(),
            u_transform: self.transform(offset, V2(1, 1), layer),
            s_texture: (self.atlas_tex.tex, sampler_info),
        };

        let draw_state = self.draw_state();
        let slice = self.meshes[idx].to_slice(gfx::TriangleList);
        let batch: Program = self.graphics.make_batch(
            &self.program, &self.meshes[idx], slice, &draw_state).unwrap();
        self.graphics.draw(&batch, &params, &self.frame);
    }

    pub fn draw_line(&mut self, p1: V2<int>, p2: V2<int>, layer: f32, thickness: f32, color: &Rgb) {
        // Use the fixed (0, 0) to (1, 1) mesh with a scale transform to
        // represent all lines.
        let sampler_info = Some(self.graphics.device.create_sampler(
            tex::SamplerInfo::new(tex::Scale, tex::Clamp)));
        let params = ShaderParam {
            u_color: color.to_array(),
            u_transform: self.transform(p1, (p2 - p1), layer),
            s_texture: (self.atlas_tex.tex, sampler_info),
        };

        let mut draw_state = self.draw_state();
        draw_state.primitive.method = gfx::state::Line(thickness);
        let slice = self.line_mesh.to_slice(gfx::Line);

        let batch: Program = self.graphics.make_batch(
            &self.program, &self.line_mesh, slice, &draw_state).unwrap();
        self.graphics.draw(&batch, &params, &self.frame);
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
                            match glfw_key::translate(k).map(|k| {
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
                let delta = t - self.last_render_time;
                let sensitivity = 0.25f64;
                self.render_duration = (1f64 - sensitivity) * self.render_duration + sensitivity * delta;

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

    uniform vec4 u_color;
    uniform mat4 u_transform;

    attribute vec3 a_pos;
    attribute vec2 a_tex_coord;

    varying vec2 v_tex_coord;
    varying vec4 v_color;

    void main() {
        v_tex_coord = a_tex_coord;
        v_color = u_color;
        gl_Position = u_transform * vec4(a_pos, 1.0);
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
    pub u_color: [f32, ..4],
    pub u_transform: [[f32, ..4], ..4],
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

type Transform = [[f32, ..4], ..4];

fn transform(scale: V2<f32>, offset: V2<f32>, z: f32) -> Transform {
    [[scale.0,  0.0,      0.0, 0.0],
     [0.0,      scale.1,  0.0, 0.0],
     [0.0,      0.0,      1.0, 0.0],
     [offset.0, offset.1, z,   1.0]]
}

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
