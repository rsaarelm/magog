extern mod extra;
extern mod cgmath;
extern mod glutil;
extern mod calx;

use glutil::app::App;
use cgmath::aabb::{Aabb2};
use cgmath::point::{Point};
use cgmath::vector::{Vec2, Vec4};
use calx::rectutil::RectUtil;
use extra::test::BenchHarness;

// Have a global app instance to keep the benchmark function from doing
// horrible window flickering.
static mut g_app: Option<App> = None;

// Run the binary with --bench command line option to run benchmark.
#[bench]
fn bench_fill(b: &mut BenchHarness) {
    unsafe {
        if g_app.is_none() {
            g_app = Some(App::new(640, 360, "fill benchmark"));
        }
    }
    let app = unsafe { g_app.get_mut_ref() };
    b.iter(|| {
        app.set_color(&Vec4::new(0.4f32, 0.0f32, 0.0f32, 1f32));
        app.fill_rect(&RectUtil::new(0.0f32, 0.0f32, 640.0f32, 360.0f32));
        app.set_color(&Vec4::new(0.8f32, 0.3f32, 0.3f32, 1f32));
        let area : Aabb2<f32> = RectUtil::new(0.0f32, 0.0f32, 213.0f32, 120.0f32);
        for p in area.points() {
            app.fill_rect(&Aabb2::new(
                    p.mul_s(3f32),
                    p.mul_s(3f32).add_v(&Vec2::new(2f32, 2f32))));
        }
        app.flush();
    });
}
