use std::mem::swap;
use std::vec;
use gl;
use color::rgb;
use color::rgb::{ToRGB};
use cgVector = cgmath::vector::Vector;
use cgmath::vector::{Vec2, Vec4};
use cgmath::point::{Point, Point2};
use cgmath::aabb::{Aabb, Aabb2};
use hgl::{Program};
use hgl;
use glfw;
use calx::rectutil::RectUtil;
use stb::image::Image;
use atlas::{Sprite, Atlas};
use recter::Recter;
use recter;
use texture::Texture;
use key;

static COLORED_V: &'static str =
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
static ALPHA_SPRITE_F: &'static str =
    "#version 130
    uniform sampler2D textureUnit;
    in vec2 texcoord;
    in vec4 color;

    void main(void) {
        float a = texture(textureUnit, texcoord).w;

        // Color key is gray value 128. This lets us
        // use both blacks and whites in the actual sprites,
        // and keeps the sprite bitmaps easy to read visually.
        // XXX: Looks like I can't do exact compare
        // with colors converted to float.
        // XXX: Also hardcoding this right into the
        // shader is a bit gross.
        if (abs(a * 255 - 128.0) <= 0.1) discard;

        gl_FragColor = vec4(color.x * a, color.y * a, color.z * a, 1.0);
    }
    ";

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

static FONT_WIDTH: f32 = 8.0;
static FONT_START_CHAR: uint = 32;
static FONT_NUM_CHARS: uint = 96;
pub static FONT_HEIGHT: f32 = 8.0;
pub static FONT_SPACE: f32 = FONT_WIDTH;

pub static SPRITE_INDEX_START: uint = FONT_NUM_CHARS + 1;

// TODO: Make a proper type.
pub type Color = Vec4<f32>;

#[deriving(Clone, Eq)]
pub struct KeyEvent {
    // Scancode (ignores local layout)
    code: uint,
    // Printable character (if any)
    ch: Option<char>,
}


#[deriving(Eq, Clone)]
pub struct MouseState {
    pos: Point2<f32>,
    left: bool,
    middle: bool,
    right: bool,
}

pub struct App {
    resolution: Vec2<f32>,
    window: ~glfw::Window,
    alive: bool,
    atlas: ~Atlas,
    sprite_shader: ~Program,
    blit_shader: ~Program,
    recter: Recter,
    key_buffer: ~[KeyEvent],
    // Key input hack flag.
    unknown_key: bool,
}

impl App {
    pub fn new(width: uint, height: uint, title: &str) -> App {
        if !glfw::init().is_ok() {
            fail!("Failed to initialize GLFW");
        }

        let window = glfw::Window::create(width as u32, height as u32, title, glfw::Windowed)
            .expect("Failed to create GLFW window.");
        window.make_context_current();
        window.set_key_polling(true);
        window.set_char_polling(true);

        gl::load_with(glfw::get_proc_address);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LEQUAL);

        gl::Viewport(0, 0, width as i32, height as i32);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        let mut ret = App {
            resolution: Vec2::new(width as f32, height as f32),
            window: ~window,
            alive: true,
            atlas: ~Atlas::new(),
            sprite_shader:
                ~Program::link(
                    [hgl::Shader::compile(COLORED_V, hgl::VertexShader).unwrap(),
                     hgl::Shader::compile(ALPHA_SPRITE_F, hgl::FragmentShader).unwrap()]
                 ).unwrap(),
            blit_shader:
                ~Program::link(
                    [hgl::Shader::compile(BLIT_V, hgl::VertexShader).unwrap(),
                     hgl::Shader::compile(BLIT_F, hgl::FragmentShader).unwrap()]
                 ).unwrap(),
            recter: Recter::new(),
            key_buffer: ~[],
            unknown_key: false,
        };

        // Hack for solid rectangles, push a solid single-pixel sprite in.
        // Assume this'll end up as position 0.
        ret.atlas.push(~Sprite::new_alpha(
                RectUtil::new(0, 0, 1, 1),
                ~[255u8]));

        let font = Image::load("assets/font.png", 1).unwrap();
        let sprites = Sprite::new_alpha_set(
            &Vec2::new(FONT_WIDTH as int, FONT_HEIGHT as int),
            &Vec2::new(font.width as int, font.height as int),
            font.pixels,
            &Vec2::new(0, -FONT_HEIGHT as int));
        for i in range(0, FONT_NUM_CHARS) {
            ret.add_sprite(~sprites[i].clone());
        }

        ret
    }

    pub fn add_sprite(&mut self, sprite: ~Sprite) -> uint {
        self.atlas.push(sprite)
    }

    pub fn draw_string<C: ToRGB>(&mut self, offset: &Vec2<f32>, color: &C, text: &str) {
        let first_font_idx = 1;

        let mut offset = *offset;
        for c in text.chars() {
            let i = c as u32;
            if i >= FONT_START_CHAR as u32
                && i < (FONT_START_CHAR + FONT_NUM_CHARS) as u32 {
                let spr = self.atlas.get(
                    (first_font_idx + i) as uint - FONT_START_CHAR);
                self.recter.add(
                    &transform_pixel_rect(&self.resolution, &spr.bounds.add_v(&offset)), 0f32,
                    &spr.texcoords, color, 1f32);
                offset.add_self_v(&Vec2::new(FONT_WIDTH, 0.0));
            }
        }
    }

    pub fn string_bounds(&mut self, text: &str) -> Aabb2<f32> {
        RectUtil::new(0f32, 0f32, text.len() as f32 * FONT_WIDTH, -FONT_HEIGHT)
    }

    pub fn fill_rect<C: ToRGB>(&mut self, rect: &Aabb2<f32>, color: &C) {
        let magic_solid_texture_index = 0;
        self.recter.add(
            &transform_pixel_rect(&self.resolution, rect), 0f32,
            &self.atlas.get(magic_solid_texture_index).texcoords,
            color, 1f32);
    }

    pub fn draw_sprite<C: ToRGB>(&mut self, idx: uint, pos: &Point2<f32>, z: f32, color: &C) {
        let spr = self.atlas.get(idx);
        self.recter.add(
            &transform_pixel_rect(&self.resolution, &spr.bounds.add_v(&pos.to_vec())), z,
            &spr.texcoords, color, 1f32);
    }

    fn render_screen_tex(&mut self) -> Texture {
        let screen_tex = Texture::new_rgba(
            self.resolution.x as uint, self.resolution.y as uint,
            Some(vec::from_elem(
                    (self.resolution.x * self.resolution.y) as uint * 4, 128u8)
                .as_slice()));
        screen_tex.render_to(|| {
            gl::Viewport(0, 0, self.resolution.x as i32, self.resolution.y as i32);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            self.atlas.bind();
            self.recter.render(self.sprite_shader);
        });
        screen_tex
    }

    pub fn flush(&mut self) {
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        // Render-to-texture.
        let screen_tex = ~self.render_screen_tex();
        self.recter.clear();

        let (width, height) = self.window.get_size();
        // XXX Odd dimensions are bad mojo for pixel perfection.
        let (width, height) = (width & !1, height & !1);
        gl::Viewport(0, 0, width, height);

        screen_tex.bind();

        let mut screen_draw = Recter::new();
        let mut bound = recter::screen_bound(&self.resolution, &Vec2::new(width as f32, height as f32));
        // XXX: Degenerate the rectangle to flip y-axis.
        swap(&mut bound.min.y, &mut bound.max.y);
        screen_draw.add(
            &bound, 0f32, &RectUtil::new(0f32, 0f32, 1f32, 1f32), &rgb::consts::WHITE, 1f32);
        screen_draw.render(self.blit_shader);

        self.window.swap_buffers();

        glfw::poll_events();

        // XXX: Dance around the borrow checker...
        let mut queue = ~[];
        for event in self.window.flush_events() {
            queue.push(event);
        }
        for &event in queue.iter() {
            self.handle_event(event);
        }

        if self.window.should_close() {
            self.alive = false
        }
    }

    pub fn screenshot(&mut self, path: &str) {
        let screen_tex = ~self.render_screen_tex();
        let bytes = screen_tex.get_bytes();
        let mut img = Image::new(self.resolution.x as uint, self.resolution.y as uint, 4);
        img.pixels = bytes;
        img.save_png(path);
    }

    fn handle_event(&mut self, (_time, event): (f64, glfw::WindowEvent)) {
        match event {
            glfw::CharEvent(ch) => {
                if !self.unknown_key {
                    if self.key_buffer.len() > 0 {
                        self.key_buffer[self.key_buffer.len() - 1].ch = Some(ch);
                    } else {
                        println!("WARNING: Received char event with no preceding key down event");
                    }
                } else {
                    // Char emitted from a key which App did not recognize.
                    // Emit the print event with code UNKNOWN.
                    self.key_buffer.push(
                        KeyEvent{ code: key::UNKNOWN, ch: Some(ch) });
                }
            },
            glfw::KeyEvent(key, _scan, action, _mods) => {
                if action == glfw::Press || action == glfw::Repeat {
                    match key::translate_glfw_key(key) {
                        Some(key) => {
                            self.key_buffer.push(KeyEvent{ code: key, ch: None });
                            self.unknown_key = false;
                        },
                        None => {
                            self.unknown_key = true;
                        }
                    };
                }
            },
            _ => ()
        };
    }

    pub fn key_buffer(&mut self) -> ~[KeyEvent] {
        let mut ret : ~[KeyEvent] = ~[];
        swap(&mut ret, &mut self.key_buffer);
        ret
    }

    pub fn get_mouse(&self) -> MouseState {
        let (cx, cy) = self.window.get_cursor_pos();
        // XXX: overly complex juggling back and forth the coordinate systems.
        let (width, height) = self.window.get_size();
        let area = Vec2::new(width as f32, height as f32);
        let bounds =
            recter::screen_bound(&self.resolution, &area)
            .add_v(&Vec2::new(1f32, 1f32))
            .mul_s(0.5f32)
            .mul_v(&area);

        MouseState {
            pos: Point2::new(
                     (cx as f32 - bounds.min.x) * (self.resolution.x / bounds.dim().x),
                     (cy as f32 - bounds.min.y) * (self.resolution.y / bounds.dim().y)),
            left: self.window.get_mouse_button(glfw::MouseButtonLeft) != glfw::Release,
            middle: self.window.get_mouse_button(glfw::MouseButtonMiddle) != glfw::Release,
            right: self.window.get_mouse_button(glfw::MouseButtonRight) != glfw::Release,
        }
    }

    pub fn screen_area(&self) -> Aabb2<f32> {
        RectUtil::new(0f32, 0f32, self.resolution.x as f32, self.resolution.y as f32)
    }

    pub fn quit(&mut self) {
        self.alive = false;
    }
}

fn transform_pixel_rect(dim: &Vec2<f32>, rect: &Aabb2<f32>) -> Aabb2<f32> {
    Aabb2::new(
        Point2::new(
            rect.min.x / dim.x * 2.0f32 - 1.0f32,
            rect.min.y / dim.y * 2.0f32 - 1.0f32),
        Point2::new(
            rect.max.x / dim.x * 2.0f32 - 1.0f32,
            rect.max.y / dim.y * 2.0f32 - 1.0f32))
}
