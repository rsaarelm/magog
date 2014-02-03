use opengles::gl2;
use cgmath::vector::{Vec2, Vec4};
use glfw;
use texture::Texture;
use shader::Shader;
use fonter::Fonter;
use stb;

static VERTEX_SHADER: &'static str =
    "#version 130
    in vec3 in_pos;
    in vec2 in_texcoord;

    uniform sampler2D texture;
    out vec2 texcoord;

    void main(void) {
        texcoord = in_texcoord;
        gl_Position = vec4(in_pos, 1.0);
    }
    ";

static FRAGMENT_SHADER: &'static str =
    "#version 130
    uniform sampler2D texture;
    in vec2 texcoord;

    void main(void) {
        vec4 col = texture2D(texture, texcoord);
        gl_FragColor = vec4(1, 1, 1, col.w);
    }
    ";

static FONT_DATA: &'static [u8] = include!("../../gen/font_data.rs");
static FONT_SIZE: f32 = 13.0;
static FONT_START_CHAR: uint = 32;
static FONT_NUM_CHARS: uint = 95;

// TODO: Make a proper type.
type Color = Vec4<u8>;

pub struct App {
    resolution: Vec2<f32>,
    draw_color: Color,
    window: ~glfw::Window,
    alive: bool,
    fonter: ~Fonter,
    texture: Option<~Texture>,
    shader: ~Shader,
}

impl App {
    pub fn new(width: uint, height: uint, title: &str) -> App {
        if !glfw::init().is_ok() {
            fail!("Failed to initialize GLFW");
        }

        let window = glfw::Window::create(width as u32, height as u32, title, glfw::Windowed)
            .expect("Failed to create GLFW window.");
        window.make_context_current();

        gl2::enable(gl2::BLEND);
        gl2::blend_func(gl2::SRC_ALPHA, gl2::ONE_MINUS_SRC_ALPHA);

        gl2::viewport(0, 0, width as i32, height as i32);
        gl2::clear(gl2::COLOR_BUFFER_BIT | gl2::DEPTH_BUFFER_BIT);

        let truetype = stb::truetype::Font::new(FONT_DATA.to_owned()).expect("Bad FONT_DATA.");

        let ret = App {
            resolution: Vec2::new(width as f32, height as f32),
            draw_color: Vec4::new(0u8, 0u8, 0u8, 255u8),
            window: ~window,
            alive: true,
            fonter: ~Fonter::new(&truetype, FONT_SIZE, FONT_START_CHAR, FONT_NUM_CHARS),
            texture: None,
            shader: ~Shader::new(VERTEX_SHADER, FRAGMENT_SHADER),
        };

        ret.shader.bind();

        ret
    }

    pub fn set_color(&mut self, color: &Color) {
        self.draw_color = *color;
    }

    pub fn draw_string(&mut self, _pos: Vec2<f32>, _text: &str) {
        self.fonter.test(self.shader);
        // TODO: This is where we'd add rectangles of the characters to the
        // rect renderer.
    }

    // TODO: Init texture
    // TODO: Draw texture rect
    // TODO: Draw filled rect

    pub fn flush(&mut self) {
        // TODO: Recter gets rendered here.
        self.window.swap_buffers();
        if self.window.should_close() {
            self.alive = false
        }
    }
}

impl Drop for App {
   fn drop(&mut self) {
        glfw::terminate();
   }
}
