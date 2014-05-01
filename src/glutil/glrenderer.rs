use std::vec::Vec;
use std::mem::swap;
use std::comm::{Receiver};
use gl;
use color::rgb;
use color::rgb::{ToRGB};
use cgVector = cgmath::vector::Vector;
use cgmath::vector::{Vector2};
use cgmath::point::{Point, Point2};
use cgmath::aabb::{Aabb, Aabb2};
use hgl::{Program};
use hgl;
use glfw;
use glfw::{Glfw, Context};
use rectutil::RectUtil;
use renderer::{Renderer, KeyEvent, MouseState, DrawMode};
use key;
use tile::Tile;
use stb::image::Image;
use glutil::atlas::{Atlas};
use glutil::recter::Recter;
use glutil::recter;
use glutil::framebuffer::Framebuffer;

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
static ALPHA_TILE_F: &'static str =
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

pub struct GlRenderer {
    pub resolution: Vector2<f32>,
    pub glfw_state: ~glfw::Glfw,
    pub window: ~glfw::Window,
    pub receiver: ~Receiver<(f64, glfw::WindowEvent)>,
    pub alive: bool,
    pub atlas: ~Atlas,
    pub tile_shader: ~Program,
    pub blit_shader: ~Program,
    pub recter: Recter,
    pub framebuffer: Framebuffer,
    pub key_buffer: Vec<KeyEvent>,
    // Key input hack flag.
    pub unknown_key: bool,
}

impl GlRenderer {
    fn render_screen(&mut self) {
        // Render-to-texture.
        self.framebuffer.bind();
        gl::Viewport(0, 0, self.resolution.x as i32, self.resolution.y as i32);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        self.atlas.bind();
        self.recter.render(self.tile_shader);
        self.framebuffer.unbind();
    }

    fn handle_event(&mut self, (_time, event): (f64, glfw::WindowEvent)) {
        match event {
            glfw::CharEvent(ch) => {
                if !self.unknown_key {
                    if self.key_buffer.len() > 0 {
                        let last_idx = self.key_buffer.len() - 1;
                        self.key_buffer.get_mut(last_idx).ch = Some(ch);
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

}

impl Renderer for GlRenderer {
    fn new(width: uint, height: uint, title: &str) -> GlRenderer {
        let glfw_state = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

        let (window, receiver) = glfw_state.create_window(
            width as u32, height as u32, title, glfw::Windowed)
            .expect("Failed to create GLFW window.");
        window.make_current();
        window.set_key_polling(true);
        window.set_char_polling(true);

        gl::load_with(|s| glfw_state.get_proc_address(s));
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        gl::Enable(gl::DEPTH_TEST);
        gl::DepthFunc(gl::LEQUAL);

        gl::Viewport(0, 0, width as i32, height as i32);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        let mut ret = GlRenderer {
            resolution: Vector2::new(width as f32, height as f32),
            glfw_state: ~glfw_state,
            window: ~window,
            receiver: ~receiver,
            alive: true,
            atlas: ~Atlas::new(),
            tile_shader:
                ~Program::link(
                    [hgl::Shader::compile(COLORED_V, hgl::VertexShader).unwrap(),
                     hgl::Shader::compile(ALPHA_TILE_F, hgl::FragmentShader).unwrap()]
                 ).unwrap(),
            blit_shader:
                ~Program::link(
                    [hgl::Shader::compile(BLIT_V, hgl::VertexShader).unwrap(),
                     hgl::Shader::compile(BLIT_F, hgl::FragmentShader).unwrap()]
                 ).unwrap(),
            recter: Recter::new(),
            framebuffer: Framebuffer::new(width, height),
            key_buffer: vec!(),
            unknown_key: false,
        };

        // Hack for solid rectangles, push a solid single-pixel tile in.
        // Assume this'll end up as position 0.
        ret.atlas.push(~Tile::new_alpha(
                RectUtil::new(0, 0, 1, 1),
                vec!(255u8)));


        ret
    }

    fn add_tile(&mut self, tile: ~Tile) -> uint {
        self.atlas.push(tile)
    }

    fn fill_rect<C: ToRGB>(&mut self, rect: &Aabb2<f32>, z: f32, color: &C) {
        let magic_solid_texture_index = 0;
        self.recter.add(
            &transform_pixel_rect(&self.resolution, rect), z,
            &self.atlas.get(magic_solid_texture_index).texcoords,
            color, 1f32);
    }

    fn draw_tile<C: ToRGB>(&mut self, idx: uint, pos: &Point2<f32>, z: f32, color: &C, _mode: DrawMode) {
        // TODO: Handle mode
        let spr = self.atlas.get(idx);
        self.recter.add(
            &transform_pixel_rect(&self.resolution, &spr.bounds.add_v(&pos.to_vec())), z,
            &spr.texcoords, color, 1f32);
    }

    fn flush(&mut self) {
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        self.render_screen();
        self.recter.clear();

        let (width, height) = self.window.get_size();
        // XXX Odd dimensions are bad mojo for pixel perfection.
        let (width, height) = (width & !1, height & !1);
        gl::Viewport(0, 0, width, height);

        self.framebuffer.texture.bind();
        let mut screen_draw = Recter::new();
        let mut bound = recter::screen_bound(&self.resolution, &Vector2::new(width as f32, height as f32));
        // XXX: Degenerate the rectangle to flip y-axis.
        swap(&mut bound.min.y, &mut bound.max.y);
        screen_draw.add(
            &bound, 0f32, &RectUtil::new(0f32, 0f32, 1f32, 1f32), &rgb::consts::WHITE, 1f32);
        screen_draw.render(self.blit_shader);

        self.window.swap_buffers();

        self.glfw_state.poll_events();

        // XXX: Dance around the borrow checker...
        let mut queue = vec!();
        for event in glfw::flush_messages(self.receiver) {
            queue.push(event);
        }
        for &event in queue.iter() {
            self.handle_event(event);
        }

        if self.window.should_close() {
            self.alive = false
        }
    }

    fn screenshot(&mut self, path: &str) {
        self.render_screen();
        let bytes = self.framebuffer.get_bytes();
        let mut img = Image::new(self.resolution.x as uint, self.resolution.y as uint, 4);
        img.pixels = bytes;
        img.save_png(path);
    }

    fn pop_key(&mut self) -> Option<KeyEvent> {
        // XXX: swap_remove seems to crash if len is 0?
        if self.key_buffer.len() == 0 {
            return None;
        }
        self.key_buffer.swap_remove(0)
    }

    fn get_mouse(&self) -> MouseState {
        let (cx, cy) = self.window.get_cursor_pos();
        // XXX: overly complex juggling back and forth the coordinate systems.
        let (width, height) = self.window.get_size();
        let area = Vector2::new(width as f32, height as f32);
        let bounds =
            recter::screen_bound(&self.resolution, &area)
            .add_v(&Vector2::new(1f32, 1f32))
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

    fn is_alive(&self) -> bool { self.alive }
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

fn translate_glfw_key(k: glfw::Key) -> Option<uint> {
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
