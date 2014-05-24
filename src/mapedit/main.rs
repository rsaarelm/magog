use std::io::{IoResult, File, Open, Write, BufferedReader, BufferedWriter};
use std::path::Path;
use serialize::json;
use serialize::{Encodable, Decodable};
use cgmath::point::{Point2};
use cgmath::vector::{Vector2};
use asciimap::AsciiMap;
use world::area::{Area, Location, ChartPos};
use world::area;
use world::transform::Transform;
use world::fov;
use world::mob::Mob;
use world::state;
use world::areaview;
use world::areaview::Kernel;
use engine::{App, Engine, Key, Image};
use engine;

pub struct State {
    area: Area,
    loc: Location,
    tiles: Vec<Image>,
    brush: uint,
}

impl state::State for State {
    fn transform(&self) -> Transform { Transform::new(ChartPos::from_location(self.loc)) }
    fn fov(&self, _loc: Location) -> fov::FovStatus { fov::Seen }
    fn drawable_mob_at<'a>(&'a self, _loc: Location) -> Option<&'a Mob> { None }
    fn area<'a>(&'a self) -> &'a Area { &self.area }
}

impl App for State {
    fn setup(&mut self, ctx: &mut Engine) {
        self.tiles = areaview::init_tiles(ctx);
    }

    fn key_pressed(&mut self, ctx: &mut Engine, key: Key) {
        match key {
            engine::KeyEscape => { ctx.quit(); }
            engine::KeyF12 => { ctx.screenshot("/tmp/shot.png"); }
            engine::Key1 => {
                self.brush += area::TERRAINS.len() - 1;
                self.brush %= area::TERRAINS.len();
            }
            engine::Key2 => {
                self.brush += 1;
                self.brush %= area::TERRAINS.len();
            }
            engine::KeyUp => { self.loc = self.loc + Vector2::new(-1, -1); }
            engine::KeyDown => { self.loc = self.loc + Vector2::new(1, 1); }
            engine::KeyLeft => { self.loc = self.loc + Vector2::new(-1, 1); }
            engine::KeyRight => { self.loc = self.loc + Vector2::new(1, -1); }
            _ => (),
        }
    }

    fn draw(&mut self, ctx: &mut Engine) {
        areaview::draw_area(ctx, &self.tiles, self);
        let mouse_pos = areaview::draw_mouse(ctx, &self.tiles, self.loc);
        let mouse = ctx.get_mouse();
        if mouse.left {
            self.area.set(mouse_pos.to_location(), area::TERRAINS[self.brush]);
        }
        if mouse.right {
            self.area.set(mouse_pos.to_location(), self.area.default);
        }

        let mut spr = areaview::SpriteCollector::new(ctx, &self.tiles);
        areaview::terrain_sprites(
            &mut spr, &Kernel::new_default(area::TERRAINS[self.brush], area::Void),
            &Point2::new(32f32, 32f32));
    }
}

impl State {
    pub fn new(area: Area) -> State {
        State {
            area: area,
            loc: Location::new(0i8, 0i8),
            tiles: vec!(),
            brush: 1,
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
        Some(State::new(Area::from_ascii_map(&ascii_map)))
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
    let mut app = State::from_file("map.txt").unwrap_or_else(||
        State::new(Area::new(area::Void)));
    Engine::run(&mut app);
    let _ = app.save("map.txt");
}
