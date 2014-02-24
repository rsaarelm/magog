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
        app.fill_rect(&RectUtil::new(0.0f32, 0.0f32, 640.0f32, 360.0f32), &consts::DARKBLUE);
        let area : Aabb2<f32> = RectUtil::new(0.0f32, 0.0f32, 213.0f32, 120.0f32);
        for p in area.points() {
            app.fill_rect(&Aabb2::new(
                    p.mul_s(3f32),
                    p.mul_s(3f32).add_v(&Vec2::new(2f32, 2f32))),
                    &consts::DARKSLATEBLUE);
        }
        app.draw_string(&Vec2::new(0f32, 8f32), &consts::SALMON, "The quick brown fox jumps over the lazy dog.");
        app.draw_string(&Vec2::new(0f32, 18f32), &consts::SALMON, "Grumpy wizards make toxic brew for the evil Queen and Jack.");
        let bounds = app.string_bounds("Painful zombies quickly watch a jinxed graveyard.").add_v(&Vec2::new(16f32, 28f32));
        app.fill_rect(&bounds, &consts::SEAGREEN);
        app.draw_string(&Vec2::new(16f32, 28f32), &consts::SALMON, "Painful zombies quickly watch a jinxed graveyard.");

        app.flush();
    }
}
