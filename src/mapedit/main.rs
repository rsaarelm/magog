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
use color::rgb::consts::*;
use cgmath::point::{Point2};
use cgmath::vector::{Vec2};
use calx::app::App;
use calx::key;
use calx::renderer::Renderer;
use calx::renderer;
use world::area::{Area, Location, TerrainType};
use world::area;
use world::transform::Transform;
use world::fov;
use world::mob::Mob;
use world::state;
use world::areaview;
use world::areaview::Kernel;
use world::sprite;

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
            area: ~Area::new(area::Void),
            pos: Location(Point2::new(0i8, 0i8)),
        }
    }
}

// XXX: Data repetition from the terrain enum.
//
// TODO: Make a terrain data system that generates both the enum and a loopable
// array from the same single terrain source. (Also the terrain properties.)
static TERRAIN_LIST: &'static [TerrainType] = &[
    area::Void,
    area::Floor,
    area::Water,
    area::Magma,
    area::Downstairs,
    area::Wall,
    area::RockWall,
    area::Rock,
    area::Tree,
    area::Grass,
    area::Stalagmite,
    area::Portal,
];

pub fn main() {
    let mut app : App<GlRenderer> = App::new(640, 360, format!("Map editor ({})", VERSION));
    areaview::init_tiles(&mut app);

    let mut state = State::new();
    let mut brush = 0;

    state.area.set(Location(Point2::new(0i8, 0i8)), area::Floor);

    while app.r.alive {
        loop {
            match app.r.pop_key() {
                Some(key) => {
                    match key.code {
                        key::ESC => { return; }
                        key::F12 => { app.r.screenshot("/tmp/shot.png"); }
                        key::NUM_1 => { brush += TERRAIN_LIST.len() - 1; brush %= TERRAIN_LIST.len(); }
                        key::NUM_2 => { brush += 1; brush %= TERRAIN_LIST.len(); }
                        key::UP => { state.pos = state.pos + Vec2::new(-1, -1); }
                        key::DOWN => { state.pos = state.pos + Vec2::new(1, 1); }
                        key::LEFT => { state.pos = state.pos + Vec2::new(-1, 1); }
                        key::RIGHT => { state.pos = state.pos + Vec2::new(1, -1); }
                        _ => (),
                    }
                }
                _ => { break; }
            }
        }

        areaview::draw_area(&state, &mut app);

        for spr in
            areaview::terrain_sprites(
                &Kernel::new_default(TERRAIN_LIST[brush], area::Void),
                &Point2::new(32f32, 32f32)).iter() {
            spr.draw(&mut app);
        }

        let mouse = app.r.get_mouse();
        let xf = Transform::new(state.pos);
        let cursor_chart_loc = xf.to_chart(&mouse.pos);
        app.r.draw_tile(areaview::CURSOR_BOTTOM, &xf.to_screen(cursor_chart_loc), sprite::FLOOR_Z, &FIREBRICK, renderer::ColorKeyDraw);
        app.r.draw_tile(areaview::CURSOR_TOP, &xf.to_screen(cursor_chart_loc), sprite::BLOCK_Z, &FIREBRICK, renderer::ColorKeyDraw);
        if mouse.left {
            state.area.set(cursor_chart_loc, TERRAIN_LIST[brush]);
        }
        if mouse.right {
            state.area.set(cursor_chart_loc, area::Void);
        }

        app.r.flush();
    }
}
