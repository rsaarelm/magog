use std::rand;
use std::mem;

use cgmath::point::{Point, Point2};
use cgmath::vector::{Vec2};
use cgmath::aabb::{Aabb, Aabb2};
use color::rgb::consts::*;

use calx::app::App;
use calx::app;
use calx::renderer::Renderer;

use area::{Location, Area, uphill};
use area;
use areaview;
use fov::Fov;
use fov;
use mapgen::MapGen;

pub struct Game {
    area: ~Area,
    pos: Location,
    seen: ~Fov,
    remembered: ~Fov,
    rng: rand::StdRng,
    stop: bool,
}

impl Game {
    pub fn new() -> Game {
        let mut ret = Game {
            area: ~Area::new(),
            pos: Location(Point2::new(0i8, 0i8)),
            seen: ~Fov::new(),
            remembered: ~Fov::new(),
            rng: rand::rng(),
            stop: true,
        };
        ret.next_level();
        ret
    }

    pub fn next_level(&mut self) {
        self.area = ~Area::new();
        self.area.gen_cave(&mut self.rng);

        self.pos = Location(Point2::new(0i8, 0i8));

        self.seen = ~Fov::new();
        self.remembered = ~Fov::new();
    }

    pub fn step(&mut self, d: &Vec2<int>) -> bool {
        let new_loc = self.pos + *d;
        if self.area.is_walkable(new_loc) {
            self.pos = new_loc;
        } else {
            return false;
        }

        if self.area.get(self.pos) == area::Downstairs {
            self.next_level();
        }

        true
    }

    pub fn draw<R: Renderer>(&mut self, app: &mut App<R>) {
        let origin = Vec2::new(320.0f32, 180.0f32);
        let mouse = app.r.get_mouse();
        let mut cursor_chart_pos = areaview::screen_to_chart(
            &mouse.pos.add_v(&origin.neg()).add_v(&Vec2::new(8.0f32, 0.0f32)));
        let Location(offset) = self.pos;
        cursor_chart_pos.x += offset.x;
        cursor_chart_pos.y += offset.y;

        let mut tmp_seen = ~fov::fov(self.area, self.pos, 12);
        mem::swap(self.seen, tmp_seen);
        // Move old fov to map memory.
        self.remembered.add(tmp_seen);

        if app.screen_area().contains(&mouse.pos) {
            if mouse.left {
                self.area.dig(Location(cursor_chart_pos));
            }

            if mouse.right {
                self.area.fill(Location(cursor_chart_pos));
            }
        }

        areaview::draw_area(self.area, app, &self.pos, self.seen, self.remembered);

        let text_zone = Aabb2::new(Point2::new(0.0f32, 200.0f32), Point2::new(240.0f32, 360.0f32));
        app.set_color(&LIGHTSLATEGRAY);
        app.print_words(&text_zone, app::Left, "Hello, player. This is a friendly status message.");

        app.set_color(&CORNFLOWERBLUE);
        app.print_words(&Aabb2::new(Point2::new(260.0f32, 0.0f32), Point2::new(380.0f32, 16.0f32)),
            app::Center, format!("cell {} {}", cursor_chart_pos.x, cursor_chart_pos.y));

        app.set_color(&LIGHTSLATEGRAY);
        app.print_words(&Aabb2::new(Point2::new(560.0f32, 0.0f32), Point2::new(640.0f32, 16.0f32)),
            app::Right, "Area Name");

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
