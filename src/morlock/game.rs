use std::rand;
use std::mem;

use cgmath::point::{Point2};
use cgmath::vector::{Vec2};
use cgmath::aabb::{Aabb, Aabb2};
use color::rgb::consts::*;

use calx::app::App;
use calx::app;
use calx::renderer::Renderer;
use calx::renderer;

use area::{Location, Area, uphill, DijkstraMap};
use area;
use areaview;
use fov::Fov;
use fov;
use mapgen::MapGen;
use mob;
use mob::Mob;
use transform::Transform;
use sprite;

// XXX: Indiscriminate blob of stuff ahoy
pub struct Game {
    area: ~Area,
    pos: Location,
    seen: ~Fov,
    remembered: ~Fov,
    mobs: ~[Mob],
    player_dijkstra: Option<DijkstraMap>,
    rng: rand::StdRng,
    stop: bool,
    depth: uint,
}

impl Game {
    pub fn new() -> Game {
        let mut ret = Game {
            area: ~Area::new(area::Rock),
            pos: Location(Point2::new(0i8, 0i8)),
            seen: ~Fov::new(),
            remembered: ~Fov::new(),
            mobs: ~[],
            player_dijkstra: None,
            rng: rand::rng(),
            stop: true,
            depth: 0,
        };
        ret.mobs.push(Mob::new(mob::Player, Location(Point2::new(0i8, 0i8))));
        ret.mobs.push(Mob::new(mob::Morlock, Location(Point2::new(4i8, 4i8))));
        ret.next_level();
        ret
    }

    pub fn player<'a>(&'a mut self) -> &'a mut Mob {
        for i in self.mobs.mut_iter() {
            if i.t == mob::Player {
                return i;
            }
        }
        fail!("No player mob");
    }

    pub fn has_player(&self) -> bool {
        for i in self.mobs.iter() {
            if i.t == mob::Player {
                return true;
            }
        }
        false
    }

    // TODO: Should have an immutable version.
    pub fn mob_at<'a>(&'a mut self, loc: Location) -> Option<&'a mut Mob> {
        for i in self.mobs.mut_iter() {
            if i.loc == loc {
                return Some(i);
            }
        }
        None
    }

    pub fn next_level(&mut self) {
        self.area = ~Area::new(area::Rock);
        self.area.gen_cave(&mut self.rng);

        self.player().loc = Location(Point2::new(0i8, 0i8));

        self.seen = ~Fov::new();
        self.remembered = ~Fov::new();
        self.depth += 1;
    }

    pub fn area_name(&self) -> ~str {
        format!("Floor {}", self.depth)
    }

    pub fn object_name(&mut self, loc: Location) -> ~str {
        match self.mob_at(loc) {
            Some(mob) => mob.data().name,
            None => ~"",
        }
    }

    pub fn step(&mut self, d: &Vec2<int>) -> bool {
        let new_loc = self.player().loc + *d;
        if self.area.is_walkable(new_loc) {
            self.player().loc = new_loc;
        } else {
            return false;
        }

        if self.area.get(new_loc) == area::Downstairs {
            self.next_level();
        }

        true
    }

    pub fn update(&mut self) {
        // TODO: Run all mobs' AI
    }

    pub fn draw<R: Renderer>(&mut self, app: &mut App<R>) {
        if self.has_player() {
            self.pos = self.player().loc;
        }

        let mouse = app.r.get_mouse();
        let xf = Transform::new(self.pos);
        let cursor_chart_loc = xf.to_chart(&mouse.pos);

        let mut tmp_seen = ~fov::fov(self.area, self.pos, 12);
        mem::swap(self.seen, tmp_seen);
        // Move old fov to map memory.
        self.remembered.add(tmp_seen);

        if app.screen_area().contains(&mouse.pos) {
            if mouse.left {
                self.area.dig(cursor_chart_loc);
            }

            if mouse.right {
                self.area.fill(cursor_chart_loc);
            }
        }

        areaview::draw_area(self, app);
        app.r.draw_tile(areaview::CURSOR_BOTTOM, &xf.to_screen(cursor_chart_loc), sprite::FLOOR_Z, &FIREBRICK, renderer::ColorKeyDraw);
        app.r.draw_tile(areaview::CURSOR_TOP, &xf.to_screen(cursor_chart_loc), sprite::BLOCK_Z, &FIREBRICK, renderer::ColorKeyDraw);

        let text_zone = Aabb2::new(Point2::new(0.0f32, 200.0f32), Point2::new(240.0f32, 360.0f32));
        app.set_color(&LIGHTSLATEGRAY);
        app.print_words(&text_zone, app::Left, "Hello, player. This is a friendly status message.");

        app.set_color(&CORNFLOWERBLUE);
        app.print_words(&Aabb2::new(Point2::new(260.0f32, 0.0f32), Point2::new(380.0f32, 16.0f32)),
            app::Center, self.object_name(cursor_chart_loc));

        app.set_color(&LIGHTSLATEGRAY);
        app.print_words(&Aabb2::new(Point2::new(560.0f32, 0.0f32), Point2::new(640.0f32, 16.0f32)),
            app::Right, self.area_name());

        if !self.stop {
            if !self.area.fully_explored(self.remembered) {
                let map = self.area.explore_map(self.remembered);
                match uphill(&map, self.pos) {
                    Some(p) => { if self.area.is_walkable(p) { self.pos = p; } },
                    None => (),
                }
            }
        }

        if self.area.get(self.pos) == area::Downstairs {
            self.next_level();
        }
    }
}
