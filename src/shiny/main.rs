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
        app.set_color(&Vec4::new(1f32, 1f32, 1f32, 1f32));
        app.fill_rect(&Aabb2::new(&Point2::new(0f32, 0f32), &Point2::new(640f32, 360f32)));
        app.set_color(&Vec4::new(0f32, 1f32, 0f32, 1f32));
        app.fill_rect(&Aabb2::new(&Point2::new(4f32, 4f32), &Point2::new(636f32, 356f32)));
        app.set_color(&Vec4::new(1f32, 0f32, 0f32, 1f32));
        app.draw_string(&Vec2::new(40f32, 40f32), "Hello, world!");
        app.flush();
    }
}
