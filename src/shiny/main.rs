#[feature(macro_rules)];

//use std::io::File;
extern mod glfw;
extern mod opengles;
extern mod cgmath;

use opengles::gl2;
use shader::Shader;
use mesh::Mesh;
use cgmath::point::Point3;

#[macro_escape]
mod gl_check;
mod shader;
mod mesh;

static VERTEX_SHADER: &'static str =
    "#version 120
    attribute vec2 v_coord;
    uniform sampler2D texture;
    varying vec2 texcoord;

    void main(void) {
	gl_Position = vec4(v_coord, 0.0, 1.0);
	texcoord = (v_coord + 1.0) / 2.0;
    }
    ";

static FRAGMENT_SHADER: &'static str =
    "#version 120
    uniform sampler2D texture;
    varying vec2 texcoord;

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

	let shader = Shader::new(VERTEX_SHADER, FRAGMENT_SHADER);
	let mesh = Mesh::new(
	    ~[Point3::new(0.0f32, 0.0f32, 0.0f32),
	      Point3::new(1.0f32, 0.0f32, 0.0f32),
	      Point3::new(0.0f32, 1.0f32, 0.0f32),
	     ],
	    ~[0, 1, 2]);

	gl2::viewport(0, 0, 800, 600);
	gl2::clear_color(0.0, 0.8, 0.8, 1.0);
	gl2::clear(gl2::COLOR_BUFFER_BIT | gl2::DEPTH_BUFFER_BIT);

	shader.bind();
	mesh.render();

	gl2::flush();

	window.swap_buffers();

        while !window.should_close() {
            glfw::poll_events();
        }
    }
}
