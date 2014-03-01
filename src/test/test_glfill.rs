extern crate extra;
extern crate cgmath;
extern crate glutil;
extern crate color;
extern crate calx;
extern crate test;

use glutil::app::App;
use cgmath::aabb::{Aabb2};
use cgmath::point::{Point};
use cgmath::vector::{Vec2};
use color::rgb;
use calx::rectutil::RectUtil;
use test::BenchHarness;

// Run the binary with --bench command line option to run benchmark.
#[bench]
fn bench_fill(b: &mut BenchHarness) {
    let mut app = App::new(640, 360, "fill benchmark");
    b.iter(|| {
        app.fill_rect(&RectUtil::new(0.0f32, 0.0f32, 640.0f32, 360.0f32), &rgb::consts::DARKVIOLET);
        let area : Aabb2<f32> = RectUtil::new(0.0f32, 0.0f32, 213.0f32, 120.0f32);
        for p in area.points() {
            app.fill_rect(&Aabb2::new(
                    p.mul_s(3f32),
                    p.mul_s(3f32).add_v(&Vec2::new(2f32, 2f32))),
                    &rgb::consts::MEDIUMVIOLETRED);
        }
        app.flush();
    });
}
