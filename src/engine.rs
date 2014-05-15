use cgmath::aabb::{Aabb, Aabb2};
use cgmath::point::{Point, Point2};
use cgmath::vector::{Vector2};
use color::rgb::consts::*;
use color::rgb::{ToRGB, RGB};
use gl::types::{GLint};
use gl;
use glfw::{Context};
use glfw;
use glutil::framebuffer::Framebuffer;
use hgl::buffer;
use hgl::texture::{Texture, ImageInfo, pixel};
use hgl::texture;
use hgl::{Program, Vao, Vbo};
use hgl;
use pack_rect::pack_rects;
use rectutil::RectUtil;
use stb::image;
use std::iter::AdditiveIterator;
use std::mem::size_of;
use std::num::{next_power_of_two};
use tile::Tile;
use timing::{Ticker, TimePerFrame};

pub trait App {
    fn setup(&mut self, ctx: &mut Engine);
    fn draw(&mut self, ctx: &mut Engine);
}

pub struct Engine {
    during_setup: bool,
    resolution: Vector2<uint>,
    title: ~str,
    // If None, render frames as fast as you can.
    frame_interval: Option<f64>,
    // Index to shader table
    // XXX: Somewhat cruddy system for pointing to a resource. Using references
    // gets us borrow checker hell.
    current_shader: uint,
    shaders: Vec<Program>,
    textures: Vec<Texture>,
    font: Vec<Image>,

    pub alive: bool,
    // Seconds per frame measurement, adjust at runtime.
    pub current_spf: f64,
    pub draw_color: RGB<u8>,
    pub background_color: RGB<u8>,
    pub z_layer: f32,
}

/// Engine is a graphics engine that runs application objects. It provides a
/// simplified interface to various graphics capabilities.
impl Engine {
    fn new() -> Engine {
        Engine {
            during_setup: true,
            resolution: Vector2::new(640u, 360u),
            title: "Application".to_owned(),
            frame_interval: None,
            current_shader: 0,
            shaders: vec!(),
            textures: vec!(),
            font: vec!(),

            alive: false,
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
        app.setup(&mut ctx);
        ctx.during_setup = false;

        let (window, _receiver) = glfw_state.create_window(
            ctx.resolution.x as u32, ctx.resolution.y as u32, ctx.title,
            glfw::Windowed)
            .expect("Failed to create GLFW window.");
        window.make_current();
        gl::load_with(|s| glfw_state.get_proc_address(s));

        // Must be in the order matching the SHADER_IDX constants.
        ctx.shaders.push(Program::link(
                [hgl::Shader::compile(BLIT_V, hgl::VertexShader).unwrap(),
                hgl::Shader::compile(BLIT_F, hgl::FragmentShader).unwrap()])
            .unwrap());
        ctx.shaders.push(Program::link(
                [hgl::Shader::compile(PLAIN_V, hgl::VertexShader).unwrap(),
                hgl::Shader::compile(PLAIN_F, hgl::FragmentShader).unwrap()])
            .unwrap());
        ctx.shaders.push(Program::link(
                [hgl::Shader::compile(TILE_V, hgl::VertexShader).unwrap(),
                hgl::Shader::compile(TILE_F, hgl::FragmentShader).unwrap()])
            .unwrap());

        // Set up texture target where the App will render into.
        let framebuffer = Framebuffer::new(ctx.resolution.x, ctx.resolution.y);

        // Turn off vsync if the user wants to go fast. Otherwise swap_buffers
        // will always clamp things to something like 60 FPS.
        //
        // XXX: Use cases where the user wants to clamp to eg. 180 FPS won't
        // work with this setup, would want a separate vsync API for those.
        if ctx.frame_interval.is_none() {
            glfw_state.set_swap_interval(0);
        }

        ctx.init_font();

        ctx.alive = true;

        // Seconds per frame meter
        let mut spf = TimePerFrame::new(0.02);
        let mut ticker = Ticker::new(
            ctx.frame_interval.unwrap_or(0.0));

        while ctx.alive {
            spf.begin();

            framebuffer.bind();
            gl::Viewport(0, 0, ctx.resolution.x as i32, ctx.resolution.y as i32);
            ctx.current_shader = TILE_SHADER_IDX;
            app.draw(&mut ctx);
            framebuffer.unbind();

            ctx.draw_screen(&framebuffer, &window);
            window.swap_buffers();

            glfw_state.poll_events();
            if window.should_close() { ctx.alive = false; }

            if ctx.frame_interval.is_some() {
                ticker.wait_for_tick();
            }

            spf.end();
            ctx.current_spf = spf.average;
        }
    }

    pub fn set_resolution(&mut self, w: uint, h: uint) {
        assert!(self.during_setup);
        self.resolution.x = w;
        self.resolution.y = h;
    }

    pub fn set_title(&mut self, title: ~str) {
        assert!(self.during_setup);
        self.title = title;
    }

    pub fn set_frame_interval(&mut self, interval_seconds: f64) {
        assert!(self.during_setup);
        assert!(interval_seconds > 0.00001);
        self.frame_interval = Some(interval_seconds);
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

    pub fn clear<C: ToRGB>(&mut self, color: &C) {
        let color = color.to_rgb::<f32>();
        gl::ClearColor(color.r, color.g, color.b, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }

    pub fn set_color<C: ToRGB>(&mut self, color: &C) {
        self.draw_color = color.to_rgb::<u8>();
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
        self.textures.get(image.texture_idx).bind();
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
    fn draw_screen(&self, framebuffer: &Framebuffer, window: &glfw::Window) {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        let (width, height) = window.get_size();
        // Clamp odd screen dimensions to even, otherwise pixel perfection
        // will break
        let (width, height) = (width & !1, height & !1);
        gl::Viewport(0, 0, width, height);

        framebuffer.texture.bind();
        let area = screen_bound(
            &Vector2::new(self.resolution.x as f32, self.resolution.y as f32),
            &Vector2::new(width as f32, height as f32));
        let vertices = Vertex::rect(
            &area, 0f32, &Point2::new(0.0f32, 1.0f32),
            &Point2::new(1.0f32, 0.0f32), &WHITE, 1f32);
        Vertex::draw_triangles(&vertices, self.shaders.get(BLIT_SHADER_IDX));
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

pub struct Image {
    texture_idx: uint,
    area: Aabb2<f32>,
    texcoords: Aabb2<f32>,
}

fn pack_tiles(tiles: &Vec<Tile>, texture_idx: uint) -> (Texture, Vec<Image>) {
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

    let info = ImageInfo::new()
        .width(base.dim().x as GLint)
        .height(base.dim().y as GLint)
        .pixel_format(pixel::RED)
        .pixel_type(pixel::UNSIGNED_BYTE)
        ;
    let texture = Texture::new_raw(texture::Texture2D);
    texture.filter(texture::Nearest);
    texture.wrap(texture::ClampToEdge);
    texture.load_image(info, tex_data.get(0));

    return (texture, images);

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
    px: f32,
    py: f32,
    pz: f32,

    u: f32,
    v: f32,

    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Vertex {
    fn new(
        px: f32, py: f32, pz: f32,
        u: f32, v: f32,
        color: &RGB<f32>, a: f32) -> Vertex {
        Vertex {
            px: px,
            py: py,
            pz: pz,
            u: u,
            v: v,
            r: color.r,
            g: color.g,
            b: color.b,
            a: a
        }
    }

    fn rect<C: ToRGB>(
        area: &Aabb2<f32>, z: f32,
        uv1: &Point2<f32>, uv2: &Point2<f32>,
        color: &C, alpha: f32) -> Vec<Vertex> {
        let c = color.to_rgb::<f32>();

        let mut ret = vec!();

        ret.push(Vertex::new(
                area.min().x, area.min().y, z,
                uv1.x, uv1.y,
                &c, alpha));

        ret.push(Vertex::new(
                area.min().x, area.max().y, z,
                uv1.x, uv2.y,
                &c, alpha));

        ret.push(Vertex::new(
                area.max().x, area.max().y, z,
                uv2.x, uv2.y,
                &c, alpha));

        ret.push(Vertex::new(
                area.min().x, area.min().y, z,
                uv1.x, uv1.y,
                &c, alpha));

        ret.push(Vertex::new(
                area.max().x, area.max().y, z,
                uv2.x, uv2.y,
                &c, alpha));

        ret.push(Vertex::new(
                area.max().x, area.min().y, z,
                uv2.x, uv1.y,
                &c, alpha));

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
