extern mod glfw;
extern mod opengles;
extern mod cgmath;
extern mod stb;
extern mod glutil;

use opengles::gl2;
use cgmath::point::{Point2, Point3};
use cgmath::aabb::{Aabb2};

use glutil::shader::Shader;
use glutil::mesh::Mesh;
use glutil::fonter::Fonter;
use std::io::File;
use std::hashmap::HashMap;

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


pub fn main() {
    do glfw::start {
        let window = glfw::Window::create(800, 600, "Shiny!", glfw::Windowed)
            .expect("Failed to create window.");
        window.make_context_current();

        let font = stb::truetype::Font::new(
            File::open(&Path::new("assets/pf_tempesta_seven_extended_bold.ttf"))
            .read_to_end())
            .unwrap();
        let fonter : Fonter<HashMap<char, Aabb2<f32>>> = Fonter::new(&font, 13.0, 32, 95);
        fonter.bind();

        gl2::enable(gl2::BLEND);
        gl2::blend_func(gl2::SRC_ALPHA, gl2::ONE_MINUS_SRC_ALPHA);

        let shader = Shader::new(VERTEX_SHADER, FRAGMENT_SHADER);
        let mesh = Mesh::new(
            ~[Point3::new(0.0f32, 0.0f32, 0.0f32),
              Point3::new(1.0f32, 0.0f32, 0.0f32),
              Point3::new(0.0f32, 1.0f32, 0.0f32),
              Point3::new(1.0f32, 1.0f32, 0.0f32),
             ],
            ~[Point2::new(0.0f32, 1.0f32),
              Point2::new(1.0f32, 1.0f32),
              Point2::new(0.0f32, 0.0f32),
              Point2::new(1.0f32, 0.0f32),
             ],
            ~[0, 1, 3, 0, 2, 3]);

        gl2::viewport(0, 0, 800, 600);
        gl2::clear_color(0.0, 0.8, 0.8, 1.0);
        gl2::clear(gl2::COLOR_BUFFER_BIT | gl2::DEPTH_BUFFER_BIT);

        shader.bind();
        gl2::uniform_1i(shader.uniform("texture").unwrap() as i32, 0);
        mesh.render(&shader);

        gl2::flush();

        window.swap_buffers();

        while !window.should_close() {
            glfw::poll_events();
        }
    }
}
