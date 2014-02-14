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

pub mod key {
    #[deriving(Clone, Eq, ToStr)]
    pub enum KeyCode {
        UP = 1,
        RIGHT = 2,
        DOWN = 3,
        LEFT = 4,
        HOME = 5,
        END = 6,
        KP5 = 7,
        BACKSPACE = 8,
        TAB = 9,
        ENTER = 10,
        PAGEUP = 11,
        PAGEDOWN = 12,
        INSERT = 13,
        DEL = 14,
        F1 = 15,
        F2 = 16,
        F3 = 17,
        F4 = 18,
        F5 = 19,
        F6 = 20,
        F7 = 21,
        F8 = 22,
        F9 = 23,
        F10 = 24,
        F11 = 25,
        F12 = 26,
        ESC = 27,
        UNKNOWN = 28,
        SPACE = 32,
        QUOTE = 39,
        ASTERISK = 42,
        PLUS = 43,
        COMMA = 44,
        MINUS = 45,
        PERIOD = 46,
        SLASH = 47,
        NUM_0 = 48,
        NUM_1 = 49,
        NUM_2 = 50,
        NUM_3 = 51,
        NUM_4 = 52,
        NUM_5 = 53,
        NUM_6 = 54,
        NUM_7 = 55,
        NUM_8 = 56,
        NUM_9 = 57,
        SEMICOLON = 59,
        EQUALS = 61,
        A = 65,
        B = 66,
        C = 67,
        D = 68,
        E = 69,
        F = 70,
        G = 71,
        H = 72,
        I = 73,
        J = 74,
        K = 75,
        L = 76,
        M = 77,
        N = 78,
        O = 79,
        P = 80,
        Q = 81,
        R = 82,
        S = 83,
        T = 84,
        U = 85,
        V = 86,
        W = 87,
        X = 88,
        Y = 89,
        Z = 90,
        LEFT_BRACKET = 91,
        BACKSLASH = 92,
        RIGHT_BRACKET = 93,
        BACKQUOTE = 96,
    }
}

fn translate_glfw_key(k: glfw::Key) -> Option<key::KeyCode> {
    match k {
        glfw::KeySpace => Some(key::SPACE),
        glfw::KeyApostrophe => Some(key::QUOTE),
        glfw::KeyComma => Some(key::COMMA),
        glfw::KeyMinus => Some(key::MINUS),
        glfw::KeyPeriod => Some(key::PERIOD),
        glfw::KeySlash => Some(key::SLASH),
        glfw::Key0 => Some(key::NUM_0),
        glfw::Key1 => Some(key::NUM_1),
        glfw::Key2 => Some(key::NUM_2),
        glfw::Key3 => Some(key::NUM_3),
        glfw::Key4 => Some(key::NUM_4),
        glfw::Key5 => Some(key::NUM_5),
        glfw::Key6 => Some(key::NUM_6),
        glfw::Key7 => Some(key::NUM_7),
        glfw::Key8 => Some(key::NUM_8),
        glfw::Key9 => Some(key::NUM_9),
        glfw::KeySemicolon => Some(key::SEMICOLON),
        glfw::KeyEqual => Some(key::EQUALS),
        glfw::KeyA => Some(key::A),
        glfw::KeyB => Some(key::B),
        glfw::KeyC => Some(key::C),
        glfw::KeyD => Some(key::D),
        glfw::KeyE => Some(key::E),
        glfw::KeyF => Some(key::F),
        glfw::KeyG => Some(key::G),
        glfw::KeyH => Some(key::H),
        glfw::KeyI => Some(key::I),
        glfw::KeyJ => Some(key::J),
        glfw::KeyK => Some(key::K),
        glfw::KeyL => Some(key::L),
        glfw::KeyM => Some(key::M),
        glfw::KeyN => Some(key::N),
        glfw::KeyO => Some(key::O),
        glfw::KeyP => Some(key::P),
        glfw::KeyQ => Some(key::Q),
        glfw::KeyR => Some(key::R),
        glfw::KeyS => Some(key::S),
        glfw::KeyT => Some(key::T),
        glfw::KeyU => Some(key::U),
        glfw::KeyV => Some(key::V),
        glfw::KeyW => Some(key::W),
        glfw::KeyX => Some(key::X),
        glfw::KeyY => Some(key::Y),
        glfw::KeyZ => Some(key::Z),
        glfw::KeyLeftBracket => Some(key::LEFT_BRACKET),
        glfw::KeyBackslash => Some(key::BACKSLASH),
        glfw::KeyRightBracket => Some(key::RIGHT_BRACKET),
        glfw::KeyGraveAccent => Some(key::BACKQUOTE),
        glfw::KeyEscape => Some(key::ESC),
        glfw::KeyEnter => Some(key::ENTER),
        glfw::KeyTab => Some(key::TAB),
        glfw::KeyBackspace => Some(key::BACKSPACE),
        glfw::KeyInsert => Some(key::INSERT),
        glfw::KeyDelete => Some(key::DEL),
        glfw::KeyRight => Some(key::RIGHT),
        glfw::KeyLeft => Some(key::LEFT),
        glfw::KeyDown => Some(key::DOWN),
        glfw::KeyUp => Some(key::UP),
        glfw::KeyPageUp => Some(key::PAGEUP),
        glfw::KeyPageDown => Some(key::PAGEDOWN),
        glfw::KeyHome => Some(key::HOME),
        glfw::KeyEnd => Some(key::END),
        glfw::KeyF1 => Some(key::F1),
        glfw::KeyF2 => Some(key::F2),
        glfw::KeyF3 => Some(key::F3),
        glfw::KeyF4 => Some(key::F4),
        glfw::KeyF5 => Some(key::F5),
        glfw::KeyF6 => Some(key::F6),
        glfw::KeyF7 => Some(key::F7),
        glfw::KeyF8 => Some(key::F8),
        glfw::KeyF9 => Some(key::F9),
        glfw::KeyF10 => Some(key::F10),
        glfw::KeyF11 => Some(key::F11),
        glfw::KeyF12 => Some(key::F12),
        glfw::KeyKp0 => Some(key::INSERT),
        glfw::KeyKp1 => Some(key::END),
        glfw::KeyKp2 => Some(key::DOWN),
        glfw::KeyKp3 => Some(key::PAGEDOWN),
        glfw::KeyKp4 => Some(key::LEFT),
        glfw::KeyKp5 => Some(key::KP5),
        glfw::KeyKp6 => Some(key::RIGHT),
        glfw::KeyKp7 => Some(key::HOME),
        glfw::KeyKp8 => Some(key::UP),
        glfw::KeyKp9 => Some(key::PAGEUP),
        glfw::KeyKpDecimal => Some(key::COMMA),
        glfw::KeyKpDivide => Some(key::SLASH),
        glfw::KeyKpMultiply => Some(key::ASTERISK),
        glfw::KeyKpSubtract => Some(key::MINUS),
        glfw::KeyKpAdd => Some(key::PLUS),
        glfw::KeyKpEnter => Some(key::ENTER),
        glfw::KeyKpEqual => Some(key::EQUALS),
        _ => None
    }
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
                    match translate_glfw_key(key) {
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
