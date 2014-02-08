extern mod glfw = "glfw-rs";
extern mod opengles;
extern mod cgmath;
extern mod stb;
extern mod glutil;

use glutil::app::App;
use cgmath::aabb::{Aabb2};
use cgmath::point::{Point2};
use cgmath::vector::{Vec2, Vec4};

pub fn main() {
    let mut app = App::new(640, 360, "Shiny!");
    while app.alive {
        glfw::poll_events();
        app.set_color(&Vec4::new(0.1f32, 0.3f32, 0.6f32, 1f32));
        app.fill_rect(
            &Aabb2::new(&Point2::new(0.0f32, 0.0f32),
            &Point2::new(640.0f32, 360.0f32)));
        app.set_color(&Vec4::new(0.5f32, 1f32, 0.5f32, 1f32));
        app.draw_string(&Vec2::new(0f32, 8f32), "The quick brown fox jumps over the lazy dog.");
        app.draw_string(&Vec2::new(0f32, 18f32), "Grumpy wizards make toxic brew for the evil Queen and Jack.");
        app.draw_string(&Vec2::new(0f32, 28f32), "Painful zombies quickly watch a jinxed graveyard.");

        app.flush();
    }
}
