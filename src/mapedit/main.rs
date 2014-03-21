#[feature(globs)];

extern crate cgmath;
extern crate glutil;
extern crate color;
extern crate calx;
extern crate stb;
extern crate collections;
extern crate num;
extern crate rand;

use glutil::glrenderer::GlRenderer;
use calx::app::App;
use calx::app;
use calx::key;
use calx::renderer::Renderer;
use calx::rectutil::RectUtil;

static VERSION: &'static str = include!("../../gen/git_version.inc");

pub fn main() {
    let mut app : App<GlRenderer> = App::new(640, 360, format!("Map editor ({})", VERSION));

    while app.r.alive {
        app.print_words(&RectUtil::new(0f32, 0f32, 640f32, 360f32), app::Left,
            "This is the map editor app. Someday it might even let you do some map editing.\n\
            For now, just press ESC to quit.");
        loop {
            match app.r.pop_key() {
                Some(key) => {
                    match key.code {
                        key::ESC => { return; },
                        key::F12 => { app.r.screenshot("/tmp/shot.png"); },
                        _ => (),
                    }
                }
                _ => { break; }
            }
        }
        app.r.flush();
    }
}
