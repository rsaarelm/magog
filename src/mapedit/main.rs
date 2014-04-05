#![feature(globs)]

extern crate cgmath;
extern crate glutil;
extern crate color;
extern crate calx;
extern crate stb;
extern crate collections;
extern crate serialize;
extern crate num;
extern crate rand;
extern crate world;
extern crate toml;

use std::io;
use std::path::Path;
use std::fmt;
use std::fmt::{Show, Formatter};
use glutil::glrenderer::GlRenderer;
use color::rgb::consts::*;
use cgmath::point::{Point2};
use cgmath::vector::{Vec2};
use calx::app::App;
use calx::key;
use calx::renderer::Renderer;
use calx::renderer;
use world::area::{Area, Location};
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
            pos: Location::new(0i8, 0i8),
        }
    }
}

// TODO: Conversion between State and StateSerialize.

// Simplified data for State that can be easily stored in TOML
// XXX: Using untyped arrays for data for brevity. Should these be subtables instead?
#[deriving(Decodable)]
struct StateSerialize {
    // Expecting exactly two elements.
    pos: ~[i8],
    // Expecting exactly two elements.
    origin: ~[i8],
    ascii_map: ~[~str],
    // Expecting exactly two elements for each legend element, with
    // the first one being exactly 1 characters long and the second
    // one being 1 or more characters.
    legend: ~[~[~str]],
}

// XXX: This is just boilerplate, would be better if TOML had Encode
// implementation and we could use that.
impl Show for StateSerialize {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        assert!(self.pos.len() == 2);
        assert!(self.origin.len() == 2);
        try!(writeln!(f.buf, "pos = [{}, {}]", self.pos[0], self.pos[1]));
        try!(writeln!(f.buf, "origin = [{}, {}]", self.origin[0], self.origin[1]));
        try!(writeln!(f.buf, "ascii_map = ["));
        for line in self.ascii_map.iter() {
            try!(writeln!(f.buf, "  \"{}\"", line));
        }
        try!(writeln!(f.buf, "]\nlegend = ["));
        for item in self.legend.iter() {
            assert!(item.len() == 2);
            assert!(item[0].len() == 1);
            assert!(item[1].len() > 0);
            try!(writeln!(f.buf, "  [{}, {}],", item.get(0), item.get(1)));
        }
        try!(writeln!(f.buf, "]"));
        Ok(())
    }
}

impl StateSerialize {
    pub fn decode<B: io::Buffer>(input: &mut B) -> Result<StateSerialize, toml::Error> {
        let value = match toml::parse_from_buffer(input) {
            Ok(v) => v,
            Err(e) => return Err(e)
        };
        toml::from_toml(value)
    }
}

pub fn main() {
    let mut app : App<GlRenderer> = App::new(640, 360, format!("Map editor ({})", VERSION));
    areaview::init_tiles(&mut app);

    let mut state = State::new();

    let mut rd = io::BufferedReader::new(io::File::open(&Path::new("map.txt")));
    let load = StateSerialize::decode(&mut rd);
    match load {
        Ok(s) => println!("{}", s),
        Err(_) => println!("Couldn't load map.txt")
    };

    let mut brush = 0;

    state.area.set(Location::new(0i8, 0i8), area::Floor);

    while app.r.alive {
        loop {
            match app.r.pop_key() {
                Some(key) => {
                    match key.code {
                        key::ESC => { return; }
                        key::F12 => { app.r.screenshot("/tmp/shot.png"); }
                        key::NUM_1 => { brush += area::TERRAINS.len() - 1; brush %= area::TERRAINS.len(); }
                        key::NUM_2 => { brush += 1; brush %= area::TERRAINS.len(); }
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
                &Kernel::new_default(area::TERRAINS[brush], area::Void),
                &Point2::new(32f32, 32f32)).iter() {
            spr.draw(&mut app);
        }

        let mouse = app.r.get_mouse();
        let xf = Transform::new(state.pos);
        let cursor_chart_loc = xf.to_chart(&mouse.pos);
        app.r.draw_tile(areaview::CURSOR_BOTTOM, &xf.to_screen(cursor_chart_loc), sprite::FLOOR_Z, &FIREBRICK, renderer::ColorKeyDraw);
        app.r.draw_tile(areaview::CURSOR_TOP, &xf.to_screen(cursor_chart_loc), sprite::BLOCK_Z, &FIREBRICK, renderer::ColorKeyDraw);
        if mouse.left {
            state.area.set(cursor_chart_loc, area::TERRAINS[brush]);
        }
        if mouse.right {
            state.area.set(cursor_chart_loc, area::Void);
        }

        app.r.flush();
    }
}
