use std::num::min;
use opengles::gl2;
use cgmath::vector::{Vector, Vec2, Vec4};
use cgmath::point::{Point2, Point3};
use cgmath::aabb::{Aabb, Aabb2};
use calx::rectutil::RectUtil;
use glfw;
use atlas::{Sprite, Atlas};
use shader::Shader;

use gl_check;

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

static FRAGMENT_SHADER: &'static str =
    "#version 130
    uniform sampler2D textureUnit;
    in vec2 texcoord;
    in vec4 color;

    void main(void) {
        gl_FragColor = vec4(
            color.x, color.y, color.z,
            color.w * texture(textureUnit, texcoord).w);
    }
    ";

static FONT_DATA: &'static [u8] = include!("../../gen/font_data.rs");
static FONT_SIZE: f32 = 13.0;
static FONT_START_CHAR: uint = 33;
static FONT_NUM_CHARS: uint = 94;

// TODO: Make a proper type.
pub type Color = Vec4<f32>;

struct Recter {
    vertices: ~[Point3<f32>],
    texcoords: ~[Point2<f32>],
    colors: ~[Color],
}

impl Recter {
    fn new() -> Recter {
        Recter {
            vertices: ~[],
            texcoords: ~[],
            colors: ~[],
        }
    }

    fn add(&mut self, area: &Aabb2<f32>, texcoords: &Aabb2<f32>, color: &Color) {
        self.vertices.push(Point3::new(area.min().x, area.min().y, 0.0));
        self.texcoords.push(Point2::new(texcoords.min().x, texcoords.min().y));
        self.colors.push(*color);

        self.vertices.push(Point3::new(area.min().x, area.max().y, 0.0));
        self.texcoords.push(Point2::new(texcoords.min().x, texcoords.max().y));
        self.colors.push(*color);

        self.vertices.push(Point3::new(area.max().x, area.max().y, 0.0));
        self.texcoords.push(Point2::new(texcoords.max().x, texcoords.max().y));
        self.colors.push(*color);


        self.vertices.push(Point3::new(area.min().x, area.min().y, 0.0));
        self.texcoords.push(Point2::new(texcoords.min().x, texcoords.min().y));
        self.colors.push(*color);

        self.vertices.push(Point3::new(area.max().x, area.max().y, 0.0));
        self.texcoords.push(Point2::new(texcoords.max().x, texcoords.max().y));
        self.colors.push(*color);

        self.vertices.push(Point3::new(area.max().x, area.min().y, 0.0));
        self.texcoords.push(Point2::new(texcoords.max().x, texcoords.min().y));
        self.colors.push(*color);

    }

    fn clear(&mut self) {
        self.vertices = ~[];
        self.texcoords = ~[];
        self.colors = ~[];
    }

    fn render(
        &mut self,
        shader: &Shader, scale: &Vec2<f32>,
        offset: &Vec2<f32>) {
        if self.vertices.len() == 0 {
            return;
        }
        // Generate buffers.
        // TODO: Wrap STREAM_DRAW buffers into RAII handles.
        let gen = gl_check!(gl2::gen_buffers(3));
        let vert = gen[0];
        let tex = gen[1];
        let col = gen[2];

        gl_check!(gl2::bind_buffer(gl2::ARRAY_BUFFER, vert));
        gl_check!(gl2::buffer_data(gl2::ARRAY_BUFFER, self.vertices, gl2::STREAM_DRAW));

        gl_check!(gl2::bind_buffer(gl2::ARRAY_BUFFER, tex));
        gl_check!(gl2::buffer_data(gl2::ARRAY_BUFFER, self.texcoords, gl2::STREAM_DRAW));

        gl_check!(gl2::bind_buffer(gl2::ARRAY_BUFFER, col));
        gl_check!(gl2::buffer_data(gl2::ARRAY_BUFFER, self.colors, gl2::STREAM_DRAW));

        // Bind shader vars.
        let in_pos = shader.attrib("in_pos").unwrap();
        gl_check!(gl2::bind_buffer(gl2::ARRAY_BUFFER, vert));
        gl_check!(gl2::vertex_attrib_pointer_f32(in_pos, 3, false, 0, 0));
        gl_check!(gl2::enable_vertex_attrib_array(in_pos));
        let in_texcoord = shader.attrib("in_texcoord").unwrap();
        gl_check!(gl2::bind_buffer(gl2::ARRAY_BUFFER, tex));
        gl_check!(gl2::vertex_attrib_pointer_f32(in_texcoord, 2, false, 0, 0));
        gl_check!(gl2::enable_vertex_attrib_array(in_texcoord));
        let in_color = shader.attrib("in_color").unwrap();
        gl_check!(gl2::bind_buffer(gl2::ARRAY_BUFFER, col));
        gl_check!(gl2::vertex_attrib_pointer_f32(in_color, 4, false, 0, 0));
        gl_check!(gl2::enable_vertex_attrib_array(in_color));

        let x = 2f32 / scale.x;
        let y = -2f32 / scale.y;
        let dx = -1f32 + 2.0 * offset.x / scale.x;
        let dy = 1f32 - 2.0 * offset.y / scale.y;
        let transform = &[
            x,    0f32, 0f32, 0f32,
            0f32, y,    0f32, 0f32,
            0f32, 0f32, 1f32, 0f32,
            dx,   dy,   0f32, 1f32,
            ];
        gl_check!(gl2::uniform_matrix_4fv(
                shader.uniform("transform").unwrap(),
                false,
                transform));

        // Draw!
        gl_check!(gl2::draw_arrays(gl2::TRIANGLES, 0, self.vertices.len() as i32));
        gl_check!(gl2::bind_buffer(gl2::ARRAY_BUFFER, 0));

        gl_check!(gl2::disable_vertex_attrib_array(in_color));
        gl_check!(gl2::disable_vertex_attrib_array(in_texcoord));
        gl_check!(gl2::disable_vertex_attrib_array(in_pos));

        self.clear();
        gl2::delete_buffers(gen);
    }
}

pub struct App {
    resolution: Vec2<f32>,
    draw_color: Color,
    window: ~glfw::Window,
    alive: bool,
    atlas: ~Atlas,
    shader: ~Shader,
    recter: Recter,
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

        //let truetype = stb::truetype::Font::new(FONT_DATA.to_owned()).expect("Bad FONT_DATA.");

        let mut ret = App {
            resolution: Vec2::new(width as f32, height as f32),
            draw_color: Vec4::new(0.5f32, 1.0f32, 0.5f32, 1.0f32),
            window: ~window,
            alive: true,
            atlas: ~Atlas::new(),
            shader: ~Shader::new(VERTEX_SHADER, FRAGMENT_SHADER),
            recter: Recter::new(),
        };

        // Hack for solid rectangles, push a solid single-pixel sprite in.
        // Assume this'll end up as position 0.
        ret.atlas.push(~Sprite::new_alpha(
                &RectUtil::new(0, 0, 1, 1),
                ~[255u8]));
        ret.atlas.push_ttf(FONT_DATA.to_owned(),
            FONT_SIZE, FONT_START_CHAR, FONT_NUM_CHARS);

        ret.shader.bind();
        ret.atlas.bind();

        ret
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

    pub fn draw_sprite(&mut self, idx: uint, offset: &Vec2<f32>) {
        let spr = self.atlas.get(idx);
        self.recter.add(
            &spr.bounds.add_v(offset),
            &spr.texcoords,
            &self.draw_color);
    }

    pub fn flush(&mut self) {
        gl2::clear(gl2::COLOR_BUFFER_BIT | gl2::DEPTH_BUFFER_BIT);
        let (width, height) = self.window.get_size();
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

        self.recter.render(
            self.shader, &Vec2::new(width as f32 / scale, height as f32 / scale),
            &offset);
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
