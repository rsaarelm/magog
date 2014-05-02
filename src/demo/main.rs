use app::App;
use cgmath::vector::{Vector2};
use color::rgb::consts::*;
use glutil::glrenderer::GlRenderer;
use key;
use rand;
use renderer::Renderer;
use renderer;
use world::area::{Area, Location, ChartPos};
use world::area;
use world::areaview;
use world::fov;
use world::mapgen::MapGen;
use world::mob::Mob;
use world::sprite;
use world::state;
use world::transform::Transform;

static VERSION: &'static str = include!("../../gen/git_version.inc");

pub struct State {
    area: Area,
    loc: Location,
}

impl state::State for State {
    fn transform(&self) -> Transform { Transform::new(ChartPos::from_location(self.loc)) }
    fn fov(&self, _loc: Location) -> fov::FovStatus { fov::Seen }
    fn drawable_mob_at<'a>(&'a self, _loc: Location) -> Option<&'a Mob> { None }
    fn area<'a>(&'a self) -> &'a Area { &self.area }
}

impl State {
    pub fn new() -> State {
        let mut area = Area::new(area::Rock);
        let mut rng = rand::StdRng::new().unwrap();
        area.gen_cave(&mut rng, false);
        State {
            area: area,
            loc: Location::new(0i8, 0i8),
        }
    }
}

pub fn main() {
    let mut app : App<GlRenderer> = App::new(640, 360, format!("Tech demo ({})", VERSION));
    areaview::init_tiles(&mut app);

    let mut state = State::new();

    while app.alive {
        loop {
            match app.r.pop_key() {
                Some(key) => {
                    match key.code {
                        key::ESC => { app.quit(); }
                        key::F12 => { app.r.screenshot("/tmp/shot.png"); }
                        key::UP => { state.loc = state.loc + Vector2::new(-1, -1); }
                        key::DOWN => { state.loc = state.loc + Vector2::new(1, 1); }
                        key::LEFT => { state.loc = state.loc + Vector2::new(-1, 1); }
                        key::RIGHT => { state.loc = state.loc + Vector2::new(1, -1); }
                        _ => (),
                    }
                }
                _ => { break; }
            }
        }

        areaview::draw_area(&state, &mut app);

        let mouse = app.r.get_mouse();
        let xf = Transform::new(ChartPos::from_location(state.loc));
        let cursor_chart_pos = xf.to_chart(&mouse.pos);
        app.r.draw_tile(areaview::CURSOR_BOTTOM, &xf.to_screen(cursor_chart_pos), sprite::FLOOR_Z, &FIREBRICK, renderer::ColorKeyDraw);
        app.r.draw_tile(areaview::CURSOR_TOP, &xf.to_screen(cursor_chart_pos), sprite::BLOCK_Z, &FIREBRICK, renderer::ColorKeyDraw);

        app.r.flush();
    }
}
