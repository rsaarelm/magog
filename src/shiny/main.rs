#[feature(globs)];
extern crate cgmath;
extern crate color;
extern crate glutil;
extern crate calx;

use glutil::glrenderer::GlRenderer;
use calx::key;
use cgmath::aabb::{Aabb2};
use cgmath::point::{Point2};
use color::rgb::consts::*;
use calx::rectutil::RectUtil;
use calx::app::App;
use calx::app;
use calx::renderer::Renderer;

static PANGRAM: &'static str =
"how quickly daft jumping zebras vex. \
SPHINX OF BLACK QUARTZ: JUDGE MY VOW. \
12345 67890 !@#$%^ &*()_+-= []{};: \"'\\ \
,./ <>?";


pub fn main() {
    let mut app : App<GlRenderer> = App::new(640, 360, "Shiny!");

    while app.alive {
        app.r.fill_rect(&RectUtil::new(0.0f32, 0.0f32, 640.0f32, 360.0f32), 0.0f32, &MIDNIGHTBLUE);

        let text_zone = Aabb2::new(Point2::new(0.0f32, 200.0f32), Point2::new(240.0f32, 360.0f32));
        app.set_color(&WHITE);
        app.print_words(&text_zone, app::Left, PANGRAM);

        app.set_color(&CORNFLOWERBLUE);
        app.print_words(&Aabb2::new(Point2::new(260.0f32, 0.0f32), Point2::new(380.0f32, 16.0f32)),
            app::Center, "Focus object");

        app.set_color(&LIGHTSLATEGRAY);
        app.print_words(&Aabb2::new(Point2::new(560.0f32, 0.0f32), Point2::new(640.0f32, 16.0f32)),
            app::Right, "Area Name");

        while app.alive {
            match app.r.pop_key() {
                Some(key) => {
                    if key.code == key::ESC {
                        return;
                    }

                    if key.code == key::F12 {
                        app.r.screenshot("/tmp/shot.png");
                    }
                },
                None => { break; },
            }
        }

        app.flush();
    }
}
