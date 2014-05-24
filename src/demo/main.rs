use cgmath::vector::{Vector2};
use color::rgb::consts::*;
use rand;
use world::area::{Area, Location, ChartPos};
use world::area;
use world::areaview;
use world::fov;
use world::mapgen::MapGen;
use world::mob::Mob;
use world::sprite;
use world::state;
use world::transform::Transform;
use engine::{App, Engine, Key, Image};
use engine;

static VERSION: &'static str = include!("../../gen/git_version.inc");

pub struct State {
    area: Area,
    loc: Location,
    tiles: Vec<Image>,
}

impl state::State for State {
    fn transform(&self) -> Transform { Transform::new(ChartPos::from_location(self.loc)) }
    fn fov(&self, _loc: Location) -> fov::FovStatus { fov::Seen }
    fn drawable_mob_at<'a>(&'a self, _loc: Location) -> Option<&'a Mob> { None }
    fn area<'a>(&'a self) -> &'a Area { &self.area }
}

impl State {
    pub fn new() -> State {
        let mut area = Area::new(area::Tree);
        let mut rng = rand::StdRng::new().unwrap();
        area.gen_herringbone(&mut rng);
        State {
            area: area,
            loc: Location::new(0i8, 0i8),
            tiles: vec!(),
        }
    }
}

impl App for State {
    fn setup(&mut self, ctx: &mut Engine) {
        self.tiles = areaview::init_tiles(ctx);
    }

    fn key_pressed(&mut self, ctx: &mut Engine, key: Key) {
        match key {
            engine::KeyEscape => { ctx.quit(); }
            engine::KeyF12 => { ctx.screenshot("/tmp/shot.png"); }
            engine::KeyUp => { self.loc = self.loc + Vector2::new(-1, -1); }
            engine::KeyDown => { self.loc = self.loc + Vector2::new(1, 1); }
            engine::KeyLeft => { self.loc = self.loc + Vector2::new(-1, 1); }
            engine::KeyRight => { self.loc = self.loc + Vector2::new(1, -1); }
            _ => (),
        }
    }

    fn draw(&mut self, ctx: &mut Engine) {
        areaview::draw_area(ctx, self, &self.tiles);

        let mouse = ctx.get_mouse();
        let xf = Transform::new(ChartPos::from_location(self.loc));
        let cursor_chart_pos = xf.to_chart(&mouse.pos);

        ctx.set_color(&FIREBRICK);
        ctx.set_layer(sprite::FLOOR_Z);
        ctx.draw_image(self.tiles.get(areaview::CURSOR_BOTTOM), &xf.to_screen(cursor_chart_pos));
        ctx.set_layer(sprite::BLOCK_Z);
        ctx.draw_image(self.tiles.get(areaview::CURSOR_TOP), &xf.to_screen(cursor_chart_pos));
    }
}

pub fn main() {
    let mut app = State::new();
    Engine::run(&mut app);
}
