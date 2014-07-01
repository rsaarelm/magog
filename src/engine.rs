use cgmath::aabb::{Aabb, Aabb2};
use cgmath::point::{Point, Point2};
use cgmath::vector::{Vector2};
use color::consts::*;
use color::{RGB};
use gl::types::{GLint, GLuint};
use gl;
use glfw::{Context};
use glfw;
use hgl::buffer;
use hgl::texture::{ImageInfo, pixel};
use hgl::texture;
use hgl::{Program, Vao, Vbo};
use hgl;
use pack_rect::pack_rects;
use rectutil::RectUtil;
use stb::image;
use std::iter::AdditiveIterator;
use std::mem;
use std::mem::size_of;
use std::num::{next_power_of_two};
use tile::Tile;
use timing::{Ticker, TimePerFrame};

pub trait App {
    fn setup(&mut self, ctx: &mut Engine);
    fn draw(&mut self, ctx: &mut Engine);

    fn char_typed(&mut self, _ctx: &mut Engine, _ch: char) {}
    fn key_pressed(&mut self, _ctx: &mut Engine, _key: Key) {}
    fn key_released(&mut self, _ctx: &mut Engine, _key: Key) {}
}

#[deriving(Clone, PartialEq, Eq)]
pub struct KeyEvent {
    // Scancode (ignores local layout)
    pub code: uint,
    // Printable character (if any)
    pub ch: Option<char>,
}

#[deriving(Clone, PartialEq)]
pub struct MouseState {
    pub pos: Point2<f32>,
    pub left: bool,
    pub middle: bool,
    pub right: bool,
}

pub struct Engine {
    alive: bool,
    resolution: Vector2<uint>,
    title: String,
    // If None, render frames as fast as you can.
    frame_interval: Option<f64>,
    window: Option<glfw::Window>,
    framebuffer: Option<Framebuffer>,
    // Index to shader table
    // XXX: Somewhat cruddy system for pointing to a resource. Using references
    // gets us borrow checker hell.
    current_shader: uint,
    shaders: Vec<Program>,
    textures: Vec<TextureProxy>,
    font: Vec<Image>,

    // Seconds per frame measurement, adjust at runtime.
    pub current_spf: f64,
    pub draw_color: RGB,
    pub background_color: RGB,
    pub z_layer: f32,
}

/// Engine is a graphics engine that runs application objects. It provides a
/// simplified interface to various graphics capabilities.
impl Engine {
    fn new() -> Engine {
        Engine {
            alive: false,
            resolution: Vector2::new(640u, 360u),
            title: "Application".to_string(),
            frame_interval: None,
            window: None,
            framebuffer: None,
            current_shader: 0,
            shaders: vec!(),
            textures: vec!(),
            font: vec!(),

            current_spf: 0.0,
            draw_color: WHITE,
            background_color: BLACK,
            z_layer: 0f32,
        }
    }

    /// Run a given application object. The application's setup method will be
    /// called at start, then the engine will loop and keep calling the
    /// application's draw method until the engine's alive flag is unset.
    pub fn run<T: App>(app: &mut T) {
        let glfw_state = glfw::init(glfw::FAIL_ON_ERRORS)
            .unwrap();

        let mut ctx = Engine::new();
        ctx.init_font();

        app.setup(&mut ctx);

        let (window, receiver) = glfw_state.create_window(
            ctx.resolution.x as u32, ctx.resolution.y as u32, ctx.title.as_slice(),
            glfw::Windowed)
            .expect("Failed to create GLFW window.");
        window.make_current();
        window.set_key_polling(true);
        window.set_char_polling(true);
        ctx.window = Some(window);

        gl::load_with(|s| glfw_state.get_proc_address(s));

        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LEQUAL);

        // Must be in the order matching the SHADER_IDX constants.
        ctx.shaders.push(Program::link(
                &[hgl::Shader::compile(BLIT_V, hgl::VertexShader),
                hgl::Shader::compile(BLIT_F, hgl::FragmentShader)])
            .unwrap());
        ctx.shaders.push(Program::link(
                &[hgl::Shader::compile(PLAIN_V, hgl::VertexShader),
                hgl::Shader::compile(PLAIN_F, hgl::FragmentShader)])
            .unwrap());
        ctx.shaders.push(Program::link(
                &[hgl::Shader::compile(TILE_V, hgl::VertexShader),
                hgl::Shader::compile(TILE_F, hgl::FragmentShader)])
            .unwrap());

        // Set up texture target where the App will render into.
        ctx.framebuffer = Some(Framebuffer::new(ctx.resolution.x, ctx.resolution.y));

        // Turn off vsync if the user wants to go fast. Otherwise swap_buffers
        // will always clamp things to something like 60 FPS.
        //
        // XXX: Use cases where the user wants to clamp to eg. 180 FPS won't
        // work with this setup, would want a separate vsync API for those.
        if ctx.frame_interval.is_none() {
            glfw_state.set_swap_interval(0);
        }

        ctx.alive = true;

        // Seconds per frame meter
        let mut spf = TimePerFrame::new(0.02);
        let mut ticker = Ticker::new(
            ctx.frame_interval.unwrap_or(0.0));

        while ctx.alive {
            spf.begin();

            ctx.get_framebuffer().bind();
            ctx.clear(&BLACK);
            gl::Viewport(0, 0, ctx.resolution.x as i32, ctx.resolution.y as i32);
            ctx.current_shader = TILE_SHADER_IDX;
            app.draw(&mut ctx);
            ctx.get_framebuffer().unbind();

            ctx.draw_screen();
            ctx.get_window().swap_buffers();

            glfw_state.poll_events();
            if ctx.get_window().should_close() { ctx.alive = false; }

            for (_time, event) in glfw::flush_messages(&receiver) {
                match event {
                    glfw::CharEvent(ch) => app.char_typed(&mut ctx, ch),
                    glfw::KeyEvent(key, _scan, action, _mods) => {
                        translate_glfw_key(key).map(
                            |key| {
                                if action == glfw::Press || action == glfw::Repeat {
                                    app.key_pressed(&mut ctx, key);
                                }
                                if action == glfw::Release {
                                    app.key_released(&mut ctx, key);
                                }
                            });
                    }
                    _ => ()
                }
            }

            if ctx.frame_interval.is_some() {
                ticker.wait_for_tick();
            }

            spf.end();
            ctx.current_spf = spf.average;
        }
    }

    pub fn set_resolution(&mut self, w: uint, h: uint) {
        assert!(!self.alive);
        self.resolution.x = w;
        self.resolution.y = h;
    }

    pub fn set_title(&mut self, title: String) {
        assert!(!self.alive);
        self.title = title;
    }

    pub fn set_frame_interval(&mut self, interval_seconds: f64) {
        assert!(!self.alive);
        assert!(interval_seconds > 0.00001);
        self.frame_interval = Some(interval_seconds);
    }

    fn get_window<'a>(&'a self) -> &'a glfw::Window {
        assert!(self.window.is_some());
        self.window.get_ref()
    }

    fn get_framebuffer<'a>(&'a self) -> &'a Framebuffer {
        assert!(self.framebuffer.is_some());
        self.framebuffer.get_ref()
    }

    /// Converts a collection of pixel data tiles into drawable images. A
    /// separate texture atlas will be alotted to each batch of images, it will
    /// be more efficient to make large batches and to batch images that will
    /// be drawn during the same frame together.
    pub fn make_images(&mut self, tiles: &Vec<Tile>) -> Vec<Image> {
        let (texture, ret) = pack_tiles(tiles, self.textures.len());
        self.textures.push(texture);
        ret
    }

    fn init_font(&mut self) {
        let font = image::Image::load_from_memory(FONT_DATA, 1).unwrap();
        let tiles = Tile::new_alpha_set(
            &Vector2::new(8, 8),
            &Vector2::new(font.width as int, font.height as int),
            font.pixels,
            &Vector2::new(0, -8));
        self.font = self.make_images(&tiles);
    }

    pub fn quit(&mut self) {
        self.alive = false;
    }

    pub fn get_mouse(&self) -> MouseState {
        let (cx, cy) = self.get_window().get_cursor_pos();
        // XXX: overly complex juggling back and forth the coordinate systems.
        let (width, height) = self.get_window().get_size();
        let area = Vector2::new(width as f32, height as f32);
        let resolution =
            Vector2::new(self.resolution.x as f32, self.resolution.y as f32);
        let bounds =
            screen_bound(&resolution, &area)
            .add_v(&Vector2::new(1f32, 1f32))
            .mul_s(0.5f32)
            .mul_v(&area);

        MouseState {
            pos: Point2::new(
                     (cx as f32 - bounds.min.x) * (resolution.x / bounds.dim().x),
                     (cy as f32 - bounds.min.y) * (resolution.y / bounds.dim().y)),
            left: self.get_window().get_mouse_button(glfw::MouseButtonLeft) != glfw::Release,
            middle: self.get_window().get_mouse_button(glfw::MouseButtonMiddle) != glfw::Release,
            right: self.get_window().get_mouse_button(glfw::MouseButtonRight) != glfw::Release,
        }
    }

    pub fn clear(&mut self, color: &RGB) {
        gl::ClearColor(color.r as f32 / 255.0, color.g as f32 / 255.0, color.b as f32 / 255.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }

    pub fn set_color(&mut self, color: &RGB) {
        self.draw_color = *color;
    }

    pub fn set_layer(&mut self, layer: f32) {
        self.z_layer = layer;
    }

    /// Draws a textured rectangle. The texture needs to have been bound before
    /// this is called. The draw_color of the Engine will be used to colorize
    /// the rectangle.
    pub fn texture_rect(
        &mut self, area: &Aabb2<f32>,
        uv1: &Point2<f32>, uv2: &Point2<f32>) {
        let area = transform_pixel_rect(
            &Vector2::new(self.resolution.x as f32, self.resolution.y as f32),
            area);
        let vertices = Vertex::rect(
            &area, self.z_layer, uv1, uv2, &self.draw_color, 1.0f32);
        Vertex::draw_triangles(&vertices, self.shaders.get(TILE_SHADER_IDX));
    }

    pub fn draw_image(&mut self, image: &Image, pos: &Point2<f32>) {
        assert!(image.texture_idx < self.textures.len(),
            "Image has nonexistent texture");
        self.textures.get_mut(image.texture_idx).bind();
        self.texture_rect(
            &image.area.add_v(&pos.to_vec()),
            image.texcoords.min(), image.texcoords.max());
    }

    pub fn draw_string(&mut self, text: &str, pos: &Point2<f32>) {
        for (x, c) in text.chars().enumerate() {
            let idx = c as i32 - 32;
            if idx >= 0 && idx < 96 {
                let img = *self.font.get(idx as uint);
                self.draw_image(
                    &img, &Point2::new(pos.x + (8 * x) as f32, pos.y));
            }
        }
    }

    /// Draw a solid-color filled rectangle.
    pub fn fill_rect(
        &mut self, area: &Aabb2<f32>) {
        let area = transform_pixel_rect(
            &Vector2::new(self.resolution.x as f32, self.resolution.y as f32),
            area);
        let vertices = Vertex::rect(
            &area, self.z_layer, &Point2::new(0f32, 0f32),
            &Point2::new(1f32, 1f32), &self.draw_color, 1.0f32);
        Vertex::draw_triangles(&vertices, self.shaders.get(PLAIN_SHADER_IDX));
    }

    /// Draw the framebuffer texture on screen.
    fn draw_screen(&self) {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        let (width, height) = self.get_window().get_size();
        // Clamp odd screen dimensions to even, otherwise pixel perfection
        // will break
        let (width, height) = (width & !1, height & !1);
        gl::Viewport(0, 0, width, height);

        self.get_framebuffer().texture.bind();
        let area = screen_bound(
            &Vector2::new(self.resolution.x as f32, self.resolution.y as f32),
            &Vector2::new(width as f32, height as f32));
        let vertices = Vertex::rect(
            &area, 0f32, &Point2::new(0.0f32, 1.0f32),
            &Point2::new(1.0f32, 0.0f32), &WHITE, 1f32);
        Vertex::draw_triangles(&vertices, self.shaders.get(BLIT_SHADER_IDX));
    }

    /// Save a png screenshot.
    pub fn screenshot(&mut self, path: &str) {
        let bytes = self.get_framebuffer().get_bytes();
        let mut img = image::Image::new(
            self.resolution.x as uint, self.resolution.y as uint, 4);
        img.pixels = bytes;
        img.save_png(path);
    }
}

static FONT_DATA: &'static [u8] = include_bin!("../assets/font.png");

static BLIT_SHADER_IDX: uint = 0;
static PLAIN_SHADER_IDX: uint = 1;
static TILE_SHADER_IDX: uint = 2;

fn screen_bound(dim: &Vector2<f32>, area: &Vector2<f32>) -> Aabb2<f32> {
    let mut scale = (area.x / dim.x).min(area.y / dim.y);
    if scale > 1.0 {
        scale = scale.floor();
    }

    let dim = Point2::new(dim.x * 2f32 * scale / area.x, dim.y * 2f32 * scale / area.y);
    let bound = Aabb2::new(Point2::new(0f32, 0f32), dim);
    bound.add_v(&Vector2::new(-dim.x / 2f32, -dim.y / 2f32))
}

fn transform_pixel_rect(dim: &Vector2<f32>, rect: &Aabb2<f32>) -> Aabb2<f32> {
    Aabb2::new(
        Point2::new(
            rect.min.x / dim.x * 2.0f32 - 1.0f32,
            rect.min.y / dim.y * 2.0f32 - 1.0f32),
        Point2::new(
            rect.max.x / dim.x * 2.0f32 - 1.0f32,
            rect.max.y / dim.y * 2.0f32 - 1.0f32))
}

#[deriving(Clone, PartialEq)]
pub struct Image {
    texture_idx: uint,
    area: Aabb2<f32>,
    texcoords: Aabb2<f32>,
}

/// Type for lazily instantiated textures.
enum TextureProxy {
    ImageData(int, int, Vec<u8>),
    Texture(texture::Texture),
}

impl TextureProxy {
    fn new(w: int, h: int, data: Vec<u8>) -> TextureProxy {
        ImageData(w, h, data)
    }

    fn bind(&mut self) {
        // Reassign self to loaded texture if it's an ImageData.
        match self {
            &ImageData(w, h, ref data) => {
                let info = ImageInfo::new()
                    .width(w as GLint)
                    .height(h as GLint)
                    .pixel_format(pixel::RED)
                    .pixel_type(pixel::UNSIGNED_BYTE)
                    ;
                let texture = texture::Texture::new_raw(texture::Texture2D);
                texture.filter(texture::Nearest);
                texture.wrap(texture::ClampToEdge);
                texture.load_image(info, data.get(0));
                texture.bind();
                Some(Texture(texture))
            }
            &Texture(ref texture) => {
                texture.bind();
                None
            }
        }.map(|x| *self = x);
    }
}

fn pack_tiles(tiles: &Vec<Tile>, texture_idx: uint) -> (TextureProxy, Vec<Image>) {
    let colorkey = 0x80u8;
    // Create gaps between the tiles to prevent edge artifacts.
    let dims = tiles.iter().map(|s| s.bounds.dim() + Vector2::new(1, 1))
        .collect::<Vec<Vector2<int>>>();
    let total_volume = dims.iter().map(|&v| v.x * v.y).sum();
    let estimate_dim = next_power_of_two((total_volume as f64).sqrt() as uint)
        as int;

    let base = RectUtil::new(0, 0, estimate_dim, estimate_dim);
    let (base, pack) = pack_rects(&base, dims.as_slice());
    // Cut off the gaps when extracting the atlas rectangles.
    let pack : Vec<Aabb2<int>> = pack.iter().map(|&rect| Aabb2::new(
            *rect.min(), rect.max().add_v(&Vector2::new(-1, -1))))
        .collect();
    assert!(pack.len() == tiles.len());

    let tex_scale = Vector2::new(
        1f32 / base.dim().x as f32, 1f32 / base.dim().y as f32);

    let mut tex_data = Vec::from_elem(base.volume() as uint, colorkey);

    let mut images = vec!();

    for i in range(0, tiles.len()) {
        paint_tile(
            tiles.get(i), &mut tex_data,
            &pack.get(i).min().to_vec(), base.dim().x);
        images.push(Image {
            texture_idx: texture_idx,
            area: to_float_rect(
                &tiles.get(i).bounds, &Vector2::new(1f32, 1f32)),
            texcoords: to_float_rect(pack.get(i), &tex_scale),
        });
    }

    return (TextureProxy::new(base.dim().x, base.dim().y, tex_data), images);

    fn paint_tile(
        tile: &Tile, tex_data: &mut Vec<u8>,
        offset: &Vector2<int>, tex_pitch: int) {
        let offset = offset - tile.bounds.min().to_vec();
        for p in tile.bounds.points() {
            tex_data.grow_set(
                (p.x + offset.x + (p.y + offset.y) * tex_pitch) as uint,
                &0, tile.at(&p));
        }
    }

    fn to_float_rect(rect: &Aabb2<int>, scale: &Vector2<f32>) -> Aabb2<f32> {
        RectUtil::new(
            rect.min().x as f32 * scale.x, rect.min().y as f32 * scale.y,
            rect.max().x as f32 * scale.x, rect.max().y as f32 * scale.y)
    }
}

struct Vertex {
    _px: f32,
    _py: f32,
    _pz: f32,

    _u: f32,
    _v: f32,

    _r: f32,
    _g: f32,
    _b: f32,
    _a: f32,
}

impl Vertex {
    fn new(
        px: f32, py: f32, pz: f32,
        u: f32, v: f32,
        color: &RGB, a: f32) -> Vertex {
        Vertex {
            _px: px,
            _py: py,
            _pz: pz,
            _u: u,
            _v: v,
            _r: color.r as f32 / 255.0,
            _g: color.g as f32 / 255.0,
            _b: color.b as f32 / 255.0,
            _a: a
        }
    }

    fn rect(
        area: &Aabb2<f32>, z: f32,
        uv1: &Point2<f32>, uv2: &Point2<f32>,
        c: &RGB, alpha: f32) -> Vec<Vertex> {

        let mut ret = vec!();

        ret.push(Vertex::new(
                area.min().x, area.min().y, z,
                uv1.x, uv1.y,
                c, alpha));

        ret.push(Vertex::new(
                area.min().x, area.max().y, z,
                uv1.x, uv2.y,
                c, alpha));

        ret.push(Vertex::new(
                area.max().x, area.max().y, z,
                uv2.x, uv2.y,
                c, alpha));

        ret.push(Vertex::new(
                area.min().x, area.min().y, z,
                uv1.x, uv1.y,
                c, alpha));

        ret.push(Vertex::new(
                area.max().x, area.max().y, z,
                uv2.x, uv2.y,
                c, alpha));

        ret.push(Vertex::new(
                area.max().x, area.min().y, z,
                uv2.x, uv1.y,
                c, alpha));

        ret
    }

    fn draw_triangles(vertices: &Vec<Vertex>, shader: &Program) {
        if vertices.len() == 0 {
            return;
        }
        let vao = Vao::new();
        let vbo = Vbo::from_data(vertices.as_slice(), buffer::StreamDraw);

        shader.bind();
        vao.bind();
        vbo.bind();

        vao.enable_attrib(
            shader, "in_pos", gl::FLOAT, 3,
            Vertex::stride(), Vertex::pos_offset());
        vao.enable_attrib(
            shader, "in_texcoord", gl::FLOAT, 2,
            Vertex::stride(), Vertex::tex_offset());
        vao.enable_attrib(
            shader, "in_color", gl::FLOAT, 4,
            Vertex::stride(), Vertex::color_offset());

        vao.draw_array(hgl::Triangles, 0, vertices.len() as i32);
    }

    fn stride() -> i32 { size_of::<Vertex>() as i32 }
    fn pos_offset() -> uint { 0 }
    fn tex_offset() -> uint { 3 * size_of::<f32>() }
    fn color_offset() -> uint { 5 * size_of::<f32>() }
}

struct Framebuffer {
    width: uint,
    height: uint,
    framebuffer: GLuint,
    depthbuffer: GLuint,
    texture: hgl::Texture,
}

impl Framebuffer {
    fn new(width: uint, height: uint) -> Framebuffer {
        let info = ImageInfo::new()
            .width(width as i32)
            .height(height as i32)
            .pixel_format(texture::pixel::RGBA)
            .pixel_type(pixel::UNSIGNED_BYTE)
            ;
        let pixels = Vec::from_elem(width * height * 4, 0u8);
        let texture = hgl::Texture::new(texture::Texture2D, info, pixels.get(0));
        texture.filter(texture::Nearest);
        texture.wrap(texture::ClampToEdge);

        let mut fb: GLuint = 0;
        unsafe { gl::GenFramebuffers(1, &mut fb); }

        // Make a depth buffer.
        let mut db: GLuint = 0;
        unsafe { gl::GenRenderbuffers(1, &mut db); }

        let ret = Framebuffer {
            width: width,
            height: height,
            framebuffer: fb,
            depthbuffer: db,
            texture: texture,
        };

        //ret.bind();
        gl::BindFramebuffer(gl::FRAMEBUFFER, ret.framebuffer);
        gl::BindRenderbuffer(gl::RENDERBUFFER, ret.depthbuffer);
        gl::FramebufferTexture2D(
            gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D, ret.texture.name, 0);
        gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH_COMPONENT, width as i32, height as i32);
        gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT,
            gl::RENDERBUFFER, ret.depthbuffer);

        assert!(
            gl::CheckFramebufferStatus(gl::FRAMEBUFFER) ==
            gl::FRAMEBUFFER_COMPLETE);

        ret.unbind();

        ret
    }

    fn bind(&self) {
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
        gl::BindRenderbuffer(gl::RENDERBUFFER, self.depthbuffer);
    }

    fn unbind(&self) {
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
    }

    fn get_bytes(&self) -> Vec<u8> {
        let ret = Vec::from_elem(self.width * self.height * 4, 0u8);
        self.texture.bind();
        unsafe {
            gl::GetTexImage(
                gl::TEXTURE_2D, 0, gl::RGBA, gl::UNSIGNED_BYTE,
                mem::transmute(ret.get(0)));
        }
        gl::BindTexture(gl::TEXTURE_2D, 0);
        ret
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            self.unbind();
            gl::DeleteFramebuffers(1, &self.framebuffer);
            gl::DeleteRenderbuffers(1, &self.depthbuffer);
        }
    }
}

static BLIT_V: &'static str =
    "#version 130
    in vec2 in_pos;
    in vec2 in_texcoord;

    out vec2 texcoord;

    void main(void) {
        texcoord = in_texcoord;
        gl_Position = vec4(in_pos, 0.0, 1.0);
    }
    ";

static BLIT_F: &'static str =
    "#version 130
    uniform sampler2D textureUnit;
    in vec2 texcoord;

    void main(void) {
        gl_FragColor = texture(textureUnit, texcoord);
    }
    ";

static PLAIN_V: &'static str =
    "#version 130
    in vec3 in_pos;
    in vec4 in_color;

    out vec4 color;

    void main(void) {
        color = in_color;
        gl_Position = vec4(in_pos, 1.0);
    }
    ";

static PLAIN_F: &'static str =
    "#version 130
    in vec4 color;

    void main(void) {
        gl_FragColor = color;
    }
    ";

static TILE_V: &'static str =
    "#version 130
    in vec3 in_pos;
    in vec2 in_texcoord;
    in vec4 in_color;

    out vec2 texcoord;
    out vec4 color;

    void main(void) {
        texcoord = in_texcoord;
        color = in_color;
        gl_Position = vec4(in_pos, 1.0);
    }
    ";

// Allow opaque shading: All alpha values except the color key are treated as
// opaque, but they also modulate RGB luminance so low alpha means a darker
// shade.
static TILE_F: &'static str =
    "#version 130
    uniform sampler2D textureUnit;
    in vec2 texcoord;
    in vec4 color;

    void main(void) {
        float a = texture(textureUnit, texcoord).x;

        // Color key is gray value 128. This lets us
        // use both blacks and whites in the actual tiles,
        // and keeps the tile bitmaps easy to read visually.
        // XXX: Looks like I can't do exact compare
        // with colors converted to float.
        // XXX: Also hardcoding this right into the
        // shader is a bit gross.
        if (abs(a * 255 - 128.0) <= 0.1) discard;

        gl_FragColor = vec4(color.x * a, color.y * a, color.z * a, 1.0);
    }
    ";

#[deriving(Clone, PartialEq, Eq, Show)]
pub enum Key {
    KeySpace = 2,
    KeyApostrophe = 3,
    KeyComma = 4,
    KeyMinus = 5,
    KeyPeriod = 6,
    KeySlash = 7,
    Key0 = 8,
    Key1 = 9,
    Key2 = 10,
    Key3 = 11,
    Key4 = 12,
    Key5 = 13,
    Key6 = 14,
    Key7 = 15,
    Key8 = 16,
    Key9 = 17,
    KeySemicolon = 18,
    KeyEquals = 19,
    KeyA = 20,
    KeyB = 21,
    KeyC = 22,
    KeyD = 23,
    KeyE = 24,
    KeyF = 25,
    KeyG = 26,
    KeyH = 27,
    KeyI = 28,
    KeyJ = 29,
    KeyK = 30,
    KeyL = 31,
    KeyM = 32,
    KeyN = 33,
    KeyO = 34,
    KeyP = 35,
    KeyQ = 36,
    KeyR = 37,
    KeyS = 38,
    KeyT = 39,
    KeyU = 40,
    KeyV = 41,
    KeyW = 42,
    KeyX = 43,
    KeyY = 44,
    KeyZ = 45,
    KeyLeftBracket = 46,
    KeyBackslash = 47,
    KeyRightBracket = 48,
    KeyGrave = 49,
    KeyEscape = 50,
    KeyEnter = 51,
    KeyTab = 52,
    KeyBackspace = 53,
    KeyInsert = 54,
    KeyDelete = 55,
    KeyRight = 56,
    KeyLeft = 57,
    KeyDown = 58,
    KeyUp = 59,
    KeyPageUp = 60,
    KeyPageDown = 61,
    KeyHome = 62,
    KeyEnd = 63,
    KeyCapsLock = 64,
    KeyScrollLock = 65,
    KeyNumLock = 66,
    KeyPrintScreen = 67,
    KeyPause = 68,
    KeyF1 = 69,
    KeyF2 = 70,
    KeyF3 = 71,
    KeyF4 = 72,
    KeyF5 = 73,
    KeyF6 = 74,
    KeyF7 = 75,
    KeyF8 = 76,
    KeyF9 = 77,
    KeyF10 = 78,
    KeyF11 = 79,
    KeyF12 = 80,
    KeyPad0 = 81,
    KeyPad1 = 82,
    KeyPad2 = 83,
    KeyPad3 = 84,
    KeyPad4 = 85,
    KeyPad5 = 86,
    KeyPad6 = 87,
    KeyPad7 = 88,
    KeyPad8 = 89,
    KeyPad9 = 90,
    KeyPadDecimal = 91,
    KeyPadDivide = 92,
    KeyPadMultiply = 93,
    KeyPadMinus = 94,
    KeyPadPlus = 95,
    KeyPadEnter = 96,
    KeyPadEquals = 97,
    KeyLeftShift = 98,
    KeyLeftControl = 99,
    KeyLeftAlt = 100,
    KeyLeftSuper = 101,
    KeyRightShift = 102,
    KeyRightControl = 103,
    KeyRightAlt = 104,
    KeyRightSuper = 105,
}

fn translate_glfw_key(k: glfw::Key) -> Option<Key> {
    match k {
        glfw::KeySpace => Some(KeySpace),
        glfw::KeyApostrophe => Some(KeyApostrophe),
        glfw::KeyComma => Some(KeyComma),
        glfw::KeyMinus => Some(KeyMinus),
        glfw::KeyPeriod => Some(KeyPeriod),
        glfw::KeySlash => Some(KeySlash),
        glfw::Key0 => Some(Key0),
        glfw::Key1 => Some(Key1),
        glfw::Key2 => Some(Key2),
        glfw::Key3 => Some(Key3),
        glfw::Key4 => Some(Key4),
        glfw::Key5 => Some(Key5),
        glfw::Key6 => Some(Key6),
        glfw::Key7 => Some(Key7),
        glfw::Key8 => Some(Key8),
        glfw::Key9 => Some(Key9),
        glfw::KeySemicolon => Some(KeySemicolon),
        glfw::KeyEqual => Some(KeyEquals),
        glfw::KeyA => Some(KeyA),
        glfw::KeyB => Some(KeyB),
        glfw::KeyC => Some(KeyC),
        glfw::KeyD => Some(KeyD),
        glfw::KeyE => Some(KeyE),
        glfw::KeyF => Some(KeyF),
        glfw::KeyG => Some(KeyG),
        glfw::KeyH => Some(KeyH),
        glfw::KeyI => Some(KeyI),
        glfw::KeyJ => Some(KeyJ),
        glfw::KeyK => Some(KeyK),
        glfw::KeyL => Some(KeyL),
        glfw::KeyM => Some(KeyM),
        glfw::KeyN => Some(KeyN),
        glfw::KeyO => Some(KeyO),
        glfw::KeyP => Some(KeyP),
        glfw::KeyQ => Some(KeyQ),
        glfw::KeyR => Some(KeyR),
        glfw::KeyS => Some(KeyS),
        glfw::KeyT => Some(KeyT),
        glfw::KeyU => Some(KeyU),
        glfw::KeyV => Some(KeyV),
        glfw::KeyW => Some(KeyW),
        glfw::KeyX => Some(KeyX),
        glfw::KeyY => Some(KeyY),
        glfw::KeyZ => Some(KeyZ),
        glfw::KeyLeftBracket => Some(KeyLeftBracket),
        glfw::KeyBackslash => Some(KeyBackslash),
        glfw::KeyRightBracket => Some(KeyRightBracket),
        glfw::KeyGraveAccent => Some(KeyGrave),
        glfw::KeyEscape => Some(KeyEscape),
        glfw::KeyEnter => Some(KeyEnter),
        glfw::KeyTab => Some(KeyTab),
        glfw::KeyBackspace => Some(KeyBackspace),
        glfw::KeyInsert => Some(KeyInsert),
        glfw::KeyDelete => Some(KeyDelete),
        glfw::KeyRight => Some(KeyRight),
        glfw::KeyLeft => Some(KeyLeft),
        glfw::KeyDown => Some(KeyDown),
        glfw::KeyUp => Some(KeyUp),
        glfw::KeyPageUp => Some(KeyPageUp),
        glfw::KeyPageDown => Some(KeyPageDown),
        glfw::KeyHome => Some(KeyHome),
        glfw::KeyEnd => Some(KeyEnd),
        glfw::KeyCapsLock => Some(KeyCapsLock),
        glfw::KeyScrollLock => Some(KeyScrollLock),
        glfw::KeyNumLock => Some(KeyNumLock),
        glfw::KeyPrintScreen => Some(KeyPrintScreen),
        glfw::KeyPause => Some(KeyPause),
        glfw::KeyF1 => Some(KeyF1),
        glfw::KeyF2 => Some(KeyF2),
        glfw::KeyF3 => Some(KeyF3),
        glfw::KeyF4 => Some(KeyF4),
        glfw::KeyF5 => Some(KeyF5),
        glfw::KeyF6 => Some(KeyF6),
        glfw::KeyF7 => Some(KeyF7),
        glfw::KeyF8 => Some(KeyF8),
        glfw::KeyF9 => Some(KeyF9),
        glfw::KeyF10 => Some(KeyF10),
        glfw::KeyF11 => Some(KeyF11),
        glfw::KeyF12 => Some(KeyF12),
        glfw::KeyKp0 => Some(KeyPad0),
        glfw::KeyKp1 => Some(KeyPad1),
        glfw::KeyKp2 => Some(KeyPad2),
        glfw::KeyKp3 => Some(KeyPad3),
        glfw::KeyKp4 => Some(KeyPad4),
        glfw::KeyKp5 => Some(KeyPad5),
        glfw::KeyKp6 => Some(KeyPad6),
        glfw::KeyKp7 => Some(KeyPad7),
        glfw::KeyKp8 => Some(KeyPad8),
        glfw::KeyKp9 => Some(KeyPad9),
        glfw::KeyKpDecimal => Some(KeyPadDecimal),
        glfw::KeyKpDivide => Some(KeyPadDivide),
        glfw::KeyKpMultiply => Some(KeyPadMultiply),
        glfw::KeyKpSubtract => Some(KeyPadMinus),
        glfw::KeyKpAdd => Some(KeyPadPlus),
        glfw::KeyKpEnter => Some(KeyPadEnter),
        glfw::KeyKpEqual => Some(KeyPadEquals),
        glfw::KeyLeftShift => Some(KeyLeftShift),
        glfw::KeyLeftControl => Some(KeyLeftControl),
        glfw::KeyLeftAlt => Some(KeyLeftAlt),
        glfw::KeyLeftSuper => Some(KeyLeftSuper),
        glfw::KeyRightShift => Some(KeyRightShift),
        glfw::KeyRightControl => Some(KeyRightControl),
        glfw::KeyRightAlt => Some(KeyRightAlt),
        glfw::KeyRightSuper => Some(KeyRightSuper),
        _ => None
    }
}
