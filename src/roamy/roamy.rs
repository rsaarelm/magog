use std::rand;

use cgmath::point::{Point, Point2};
use cgmath::vector::{Vec2};
use cgmath::aabb::{Aabb};
use area::{Location, Area};
use areaview;
use glutil::app::App;
use fov::Fov;
use mapgen::MapGen;

pub struct Roamy {
    area: ~Area,
    pos: Location,
    seen: ~Fov,
    remembered: ~Fov,
    rng: rand::StdRng,
}

impl Roamy {
    pub fn new() -> Roamy {
        let mut ret = Roamy {
            area: ~Area::new(),
            pos: Location(Point2::new(0i8, 0i8)),
            seen: ~Fov::new(),
            remembered: ~Fov::new(),
            rng: rand::rng(),
        };
        ret.next_level();
        ret
    }

    pub fn next_level(&mut self) {
        self.area.gen_cave(&mut self.rng);
        self.pos = Location(Point2::new(0i8, 0i8));
    }

    pub fn draw(&mut self, app: &mut App) {
        let origin = Vec2::new(320.0f32, 180.0f32);
        let mouse = app.get_mouse();
        let cursor_chart_pos = areaview::screen_to_chart(
            &mouse.pos.add_v(&origin.neg()).add_v(&Vec2::new(8.0f32, 0.0f32)));

        if app.screen_area().contains(&mouse.pos) {
            if mouse.left {
                self.area.dig(&Location(cursor_chart_pos));
            }

            if mouse.right {
                self.area.fill(&Location(cursor_chart_pos));
            }
        }

        areaview::draw_area(self.area, app, &self.pos, self.seen, self.remembered);
    }
}
