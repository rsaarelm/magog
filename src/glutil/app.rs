use std::mem::swap;
use std::num::min;
use opengles::gl2;
use cgmath::vector::{Vector, Vec2, Vec4};
use cgmath::point::{Point, Point2};
use cgmath::aabb::{Aabb, Aabb2};
use calx::rectutil::RectUtil;
use glfw;
use atlas::{Sprite, Atlas};
use shader::Shader;
use recter::Recter;
use key;

static VERTEX_SHADER: &'static str =
    "#version 130
    in vec3 in_pos;
    in vec2 in_texcoord;
    in vec4 in_color;
    uniform mat4 transform;

    out vec2 texcoord;
    out vec4 color;

    void main(void) {
        texcoord = in_texcoord;
        color = in_color;
        gl_Position = transform * vec4(in_pos, 1.0);
    }
    ";

// Allow opaque shading: All nonzero alpha values are treated as opaque, but
// they also modulate RGB luminance so low alpha means a darker shade.
static FRAGMENT_SHADER: &'static str =
    "#version 130
    uniform sampler2D textureUnit;
    in vec2 texcoord;
    in vec4 color;

    void main(void) {
        float a = texture(textureUnit, texcoord).w;
        gl_FragColor = vec4(
            color.x * a, color.y * a, color.z * a,
            a > 0 ? 1.0 : 0.0);
    }
    ";

static FONT_DATA: &'static [u8] = include!("../../gen/font_data.rs");
static FONT_SIZE: f32 = 13.0;
static FONT_START_CHAR: uint = 33;
static FONT_NUM_CHARS: uint = 94;

// TODO: Make a proper type.
pub type Color = Vec4<f32>;

#[deriving(Clone, Eq, ToStr)]
pub struct KeyEvent {
    // Scancode (ignores local layout)
    code: key::KeyCode,
    // Printable character (if any)
    ch: Option<char>,
}


#[deriving(Eq, Clone, ToStr)]
pub struct MouseState {
    pos: Point2<f32>,
    left: bool,
    middle: bool,
    right: bool,
}

pub struct App {
    resolution: Vec2<f32>,
    draw_color: Color,
    window: ~glfw::Window,
    alive: bool,
    atlas: ~Atlas,
    shader: ~Shader,
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

        gl2::enable(gl2::BLEND);
        gl2::blend_func(gl2::SRC_ALPHA, gl2::ONE_MINUS_SRC_ALPHA);

        gl2::viewport(0, 0, width as i32, height as i32);
        gl2::clear(gl2::COLOR_BUFFER_BIT | gl2::DEPTH_BUFFER_BIT);

        let mut ret = App {
            resolution: Vec2::new(width as f32, height as f32),
            draw_color: Vec4::new(0.5f32, 1.0f32, 0.5f32, 1.0f32),
            window: ~window,
            alive: true,
            atlas: ~Atlas::new(),
            shader: ~Shader::new(VERTEX_SHADER, FRAGMENT_SHADER),
            recter: Recter::new(),
            key_buffer: ~[],
            unknown_key: false,
        };

        // Hack for solid rectangles, push a solid single-pixel sprite in.
        // Assume this'll end up as position 0.
        ret.atlas.push(~Sprite::new_alpha(
                RectUtil::new(0, 0, 1, 1),
                ~[255u8]));
        ret.atlas.push_ttf(FONT_DATA.to_owned(),
            FONT_SIZE, FONT_START_CHAR, FONT_NUM_CHARS);

        ret.shader.bind();
        ret.atlas.bind();

        ret
    }

    pub fn add_sprite(&mut self, sprite: ~Sprite) -> uint {
        self.atlas.push(sprite)
    }

    pub fn set_color(&mut self, color: &Color) {
        self.draw_color = *color;
    }

    pub fn draw_string(&mut self, offset: &Vec2<f32>, text: &str) {
        let first_font_idx = 1;

        let mut offset = *offset;
        for c in text.chars() {
            let i = c as u32;
            if i == 32 {
                // XXX: Space hack.
                offset.add_self_v(&Vec2::new((FONT_SIZE / 2.0).floor(), 0.0));
            } else if i >= FONT_START_CHAR as u32
                && i < (FONT_START_CHAR + FONT_NUM_CHARS) as u32 {
                let spr = self.atlas.get(
                    (first_font_idx + i) as uint - FONT_START_CHAR);
                self.recter.add(
                    &spr.bounds.add_v(&offset),
                    &spr.texcoords,
                    &self.draw_color);
                offset.add_self_v(&Vec2::new(spr.bounds.dim().x + 1.0, 0.0));
            }
        }
    }

    pub fn fill_rect(&mut self, rect: &Aabb2<f32>) {
        let magic_solid_texture_index = 0;
        self.recter.add(
            rect,
            &self.atlas.get(magic_solid_texture_index).texcoords,
            &self.draw_color);
    }

    pub fn draw_sprite(&mut self, idx: uint, pos: &Point2<f32>) {
        let spr = self.atlas.get(idx);
        self.recter.add(
            &spr.bounds.add_v(&pos.to_vec()),
            &spr.texcoords,
            &self.draw_color);
    }

    fn scale_params(&self) -> (f32, Vec2<f32>, Vec2<f32>) {
        let (width, height) = self.window.get_size();

        // XXX: The pixel scaling routine doesn't seem to like odd window
        // dimensions. Zero the lowest bits to make them even.
        let (width, height) = (width ^ 1, height ^ 1);

        gl2::viewport(0, 0, width, height);
        let mut scale = min(
            width as f32 / self.resolution.x,
            height as f32 / self.resolution.y);
        if scale > 1.0 {
            scale = scale.floor();
        }

        let offset = Vec2::new(width as f32, height as f32)
            .sub_v(&self.resolution.mul_s(scale))
            .div_s(2.0 * scale);
        (scale, offset, Vec2::new(width as f32 / scale, height as f32 / scale))
    }

    pub fn flush(&mut self) {
        gl2::clear(gl2::COLOR_BUFFER_BIT | gl2::DEPTH_BUFFER_BIT);
        self.atlas.bind();

        let (_scale, offset, dim) = self.scale_params();

        self.recter.render(self.shader, &dim, &offset);
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
        let (scale, offset, _dim) = self.scale_params();

        MouseState {
            pos: Point2::new(cx as f32 / scale - offset.x, cy as f32 / scale - offset.y),
            left: self.window.get_mouse_button(glfw::MouseButtonLeft) != glfw::Release,
            middle: self.window.get_mouse_button(glfw::MouseButtonMiddle) != glfw::Release,
            right: self.window.get_mouse_button(glfw::MouseButtonRight) != glfw::Release,
        }
    }
}
