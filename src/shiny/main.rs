extern mod glfw = "glfw-rs";
extern mod opengles;
extern mod cgmath;
extern mod stb;
extern mod glutil;

use glutil::app::App;
use cgmath::vector::{Vec2};

pub fn main() {
    let mut app = App::new(800, 600, "Shiny!");
    app.draw_string(Vec2::new(0f32, 0f32), "Hello, world!");
    while app.alive {
        glfw::poll_events();
        app.flush();
    }
}
