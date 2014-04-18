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

use std::io::{IoResult, File, Open, Write, BufferedReader, BufferedWriter};
use std::path::Path;
use serialize::json;
use serialize::{Encodable, Decodable};
use glutil::glrenderer::GlRenderer;
use color::rgb::consts::*;
use cgmath::point::{Point2};
use cgmath::vector::{Vector2};
use calx::app::App;
use calx::key;
use calx::renderer::Renderer;
use calx::renderer;
use calx::asciimap::AsciiMap;
use world::area::{Area, Location, ChartPos};
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
    loc: Location,
}

impl state::State for State {
    fn transform(&self) -> Transform { Transform::new(ChartPos::from_location(self.loc)) }
    fn fov(&self, _loc: Location) -> fov::FovStatus { fov::Seen }
    fn drawable_mob_at<'a>(&'a self, _loc: Location) -> Option<&'a Mob> { None }
    fn area<'a>(&'a self) -> &'a Area { &*self.area }
}

impl State {
    pub fn new() -> State {
        State {
            area: ~Area::new(area::Void),
            loc: Location::new(0i8, 0i8),
        }
    }

    pub fn from_file(path: &str) -> Option<State> {
        let mut rd = BufferedReader::new(File::open(&Path::new(path)));
        let json_value = match json::from_reader(&mut rd) {
            Ok(v) => v,
            Err(e) => { println!("JSON parse error {}", e.to_str()); return None; }
        };
        let mut decoder = json::Decoder::new(json_value);
        let ascii_map: AsciiMap = match Decodable::decode(&mut decoder) {
            Ok(v) => v,
            Err(e) => { println!("Decoding error: {}", e); return None; }
        };
        Some(State {
            area: ~Area::from_ascii_map(&ascii_map),
            loc: Location::new(0i8, 0i8),
        })
    }

    pub fn save(&self, path: &str) -> IoResult<()> {
        let file = File::open_mode(&Path::new(path), Open, Write).unwrap();
        let mut wr = BufferedWriter::new(file);
        let mut encoder = json::PrettyEncoder::new(&mut wr);
        let obj = self.area.build_asciimap();
        try!(obj.encode(&mut encoder));
        Ok(())
    }
}

pub fn main() {
    let mut app : App<GlRenderer> = App::new(640, 360, format!("Map editor ({})", VERSION));
    areaview::init_tiles(&mut app);

    let mut state = match State::from_file("map.txt") {
        Some(s) => s,
        None => State::new()
    };

    let mut brush = 0;

    while app.alive {
        loop {
            match app.r.pop_key() {
                Some(key) => {
                    match key.code {
                        key::ESC => { app.quit(); }
                        key::F12 => { app.r.screenshot("/tmp/shot.png"); }
                        key::NUM_1 => { brush += area::TERRAINS.len() - 1; brush %= area::TERRAINS.len(); }
                        key::NUM_2 => { brush += 1; brush %= area::TERRAINS.len(); }
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

        for spr in
            areaview::terrain_sprites(
                &Kernel::new_default(area::TERRAINS[brush], area::Void),
                &Point2::new(32f32, 32f32)).iter() {
            spr.draw(&mut app);
        }

        let mouse = app.r.get_mouse();
        let xf = Transform::new(ChartPos::from_location(state.loc));
        let cursor_chart_pos = xf.to_chart(&mouse.pos);
        app.r.draw_tile(areaview::CURSOR_BOTTOM, &xf.to_screen(cursor_chart_pos), sprite::FLOOR_Z, &FIREBRICK, renderer::ColorKeyDraw);
        app.r.draw_tile(areaview::CURSOR_TOP, &xf.to_screen(cursor_chart_pos), sprite::BLOCK_Z, &FIREBRICK, renderer::ColorKeyDraw);
        if mouse.left {
            state.area.set(cursor_chart_pos.to_location(), area::TERRAINS[brush]);
        }
        if mouse.right {
            state.area.set(cursor_chart_pos.to_location(), state.area.default);
        }

        app.r.flush();
    }
    let _ = state.save("map.txt");
}
