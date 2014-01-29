#[feature(macro_rules)];

extern mod glfw;
extern mod opengles;
extern mod cgmath;
extern mod stb;

use opengles::gl2;
use cgmath::point::{Point2, Point3};

use shader::Shader;
use mesh::Mesh;
use texture::Texture;
use stb::image::Image;

#[macro_escape]
mod gl_check;
mod shader;
mod mesh;
mod texture;

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
	gl_FragColor = texture2D(texture, texcoord);
    }
    ";


pub fn main() {
    println!("Shiny: A prototype user interface.");
    do glfw::start {
        let window = glfw::Window::create(800, 600, "Shiny!", glfw::Windowed)
            .expect("Failed to create window.");
        window.make_context_current();

        let bitmap = Image::load("assets/texture.png", 4).unwrap();
        let texture = Texture::new_rgba(bitmap.width, bitmap.height, bitmap.pixels);
        texture.bind();

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
