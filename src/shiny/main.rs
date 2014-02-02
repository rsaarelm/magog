extern mod glfw;
extern mod opengles;
extern mod cgmath;
extern mod stb;
extern mod glutil;

use glutil::app::App;
use std::io::File;
use cgmath::vector::{Vec2};

pub fn main() {
    let mut app = App::new(800, 600, "Shiny!");
    let font_data = File::open(&Path::new("assets/pf_tempesta_seven_extended_bold.ttf"))
        .read_to_end();
    app.init_font(font_data, 13.0, 32, 95);
    app.draw_string(Vec2::new(0f32, 0f32), "Hello, world!");
    while app.alive {
        glfw::poll_events();
        app.flush();
    }
}
