use cgmath::aabb::{Aabb2};
use cgmath::point::{Point2};
use color::rgb::consts::*;
use engine::{App, Engine};

struct EngineTest {
    tick: int,
}

impl EngineTest {
    pub fn new() -> EngineTest {
        EngineTest {
            tick: 0,
        }
    }
}

impl App for EngineTest {
    fn setup(&mut self, _ctx: &mut Engine) {
        //ctx.set_frame_interval(0.033);
    }

    fn draw(&mut self, ctx: &mut Engine) {
        self.tick += 1;

        ctx.clear(&MAROON);
        ctx.set_color(&ORANGE);
        let x = (self.tick % 660) as f32;
        ctx.fill_rect(
            &Aabb2::new(
                Point2::new(1.0 + x, 1.0),
                Point2::new(16f32 + x * 2f32, 16f32 + x)));
        ctx.draw_string(
            format!("FPS: {:.0}", 1.0f64 / ctx.current_spf),
            &Point2::new(0f32, 8f32));
    }
}

pub fn main() {
    let mut app = EngineTest::new();
    Engine::run(&mut app);
}
