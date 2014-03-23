#[feature(globs)];

extern crate cgmath;
extern crate glutil;
extern crate color;
extern crate calx;
extern crate stb;
extern crate collections;
extern crate num;
extern crate rand;
extern crate world;

use glutil::glrenderer::GlRenderer;
use cgmath::point::{Point2};
use calx::app::App;
use calx::app;
use calx::key;
use calx::renderer::Renderer;
use calx::rectutil::RectUtil;
use world::area::{Area, Location};
use world::area;
use world::transform::Transform;
use world::fov;
use world::mob::Mob;
use world::state;
use world::areaview;

static VERSION: &'static str = include!("../../gen/git_version.inc");

pub struct State {
    area: ~Area,
    pos: Location,
}

impl state::State for State {
    fn transform(&self) -> Transform { Transform::new(self.pos) }
    fn fov(&self, _loc: Location) -> fov::FovStatus { fov::Seen }
    fn drawable_mob_at<'a>(&'a self, _loc: Location) -> Option<&'a Mob> { None }
    fn area<'a>(&'a self) -> &'a Area { &*self.area }
}

impl State {
    pub fn new() -> State {
        State {
            area: ~Area::new(area::Rock),
            pos: Location(Point2::new(0i8, 0i8)),
        }
    }
}

pub fn main() {
    let mut app : App<GlRenderer> = App::new(640, 360, format!("Map editor ({})", VERSION));
    areaview::init_tiles(&mut app);

    let mut state = State::new();

    state.area.set(Location(Point2::new(0i8, 0i8)), area::Floor);

    while app.r.alive {
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

        areaview::draw_area(&state, &mut app);

        app.print_words(&RectUtil::new(0f32, 0f32, 640f32, 360f32), app::Left,
            "This is the map editor app. Someday it might even let you do some map editing.\n\
            For now, just press ESC to quit.");
        app.r.flush();
    }
}
