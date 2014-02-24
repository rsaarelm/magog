extern crate cgmath;
extern crate color;
extern crate glutil;
extern crate calx;

use glutil::app::App;
use cgmath::aabb::{Aabb, Aabb2};
use cgmath::point::{Point};
use cgmath::vector::{Vec2};
use color::rgb::consts;
use calx::rectutil::RectUtil;

pub fn main() {
    let mut app = App::new(640, 360, "Shiny!");
    while app.alive {
        app.set_color(&consts::DARKBLUE);
        app.fill_rect(&RectUtil::new(0.0f32, 0.0f32, 640.0f32, 360.0f32));
        app.set_color(&consts::DARKSLATEBLUE);
        let area : Aabb2<f32> = RectUtil::new(0.0f32, 0.0f32, 213.0f32, 120.0f32);
        for p in area.points() {
            app.fill_rect(&Aabb2::new(
                    p.mul_s(3f32),
                    p.mul_s(3f32).add_v(&Vec2::new(2f32, 2f32))));
        }
        app.set_color(&consts::SALMON);
        app.draw_string(&Vec2::new(0f32, 8f32), "The quick brown fox jumps over the lazy dog.");
        app.draw_string(&Vec2::new(0f32, 18f32), "Grumpy wizards make toxic brew for the evil Queen and Jack.");
        app.set_color(&consts::SEAGREEN);
        let bounds = app.string_bounds("Painful zombies quickly watch a jinxed graveyard.").add_v(&Vec2::new(16f32, 28f32));
        app.fill_rect(&bounds);
        app.set_color(&consts::SALMON);
        app.draw_string(&Vec2::new(16f32, 28f32), "Painful zombies quickly watch a jinxed graveyard.");

        app.flush();
    }
}
