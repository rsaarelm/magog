extern crate cgmath;
extern crate glutil;
extern crate calx;
extern crate stb;

use std::hashmap::HashSet;
use std::rand;
use std::rand::Rng;

use glutil::app::App;
use glutil::key;
use glutil::atlas::Sprite;
use cgmath::aabb::{Aabb, Aabb2};
use cgmath::point::{Point, Point2};
use cgmath::vector::{Vec2, Vec4};
use calx::rectutil::RectUtil;
use stb::image::Image;
use calx::text::Map2DUtil;
use area::Area;
use area::{is_solid, DIRECTIONS, Location};

pub mod fov;
pub mod area;
pub mod areaview;
pub mod dijkstra;

pub trait MapGen {
    fn gen_cave<R: Rng>(&mut self, rng: &mut R);
    fn gen_prefab(&mut self, prefab: &str);
}

impl MapGen for Area {
    fn gen_cave<R: Rng>(&mut self, rng: &mut R) {
        let center = Location(Point2::new(0i8, 0i8));
        let mut edge = HashSet::new();
        let bounds = Aabb2::new(Point2::new(-16i8, -16i8), Point2::new(16i8, 16i8));
        let mut dug = 1;
        self.dig(&center);
        for &v in DIRECTIONS.iter() {
            edge.insert(center + v);
        }

        for _itercount in range(0, 10000) {
            let pick = *rng.sample(edge.iter(), 1)[0];
            let nfloor = DIRECTIONS.iter().count(|&v| self.is_open(&(pick + v)));
            assert!(nfloor > 0);

            // Weight digging towards narrow corners.
            if rng.gen_range(0, nfloor * nfloor) != 0 {
                continue;
            }

            self.dig(&pick);
            dug += 1;

            for &v in DIRECTIONS.iter() {
                let p = pick + v;
                if !self.defined(&p) && bounds.contains(p.p()) {
                    edge.insert(p);
                }
            }

            if dug > 384 { break; }
        }
    }

    fn gen_prefab(&mut self, prefab: &str) {
        for (c, x, y) in prefab.chars().map2d() {
            if c == '.' {
                self.set.insert(Location(Point2::new(x as i8, y as i8)), area::Floor);
            }
            if c == '~' {
                self.set.insert(Location(Point2::new(x as i8, y as i8)), area::Water);
            }
        }

    }
}

pub fn main() {
    let mut app = App::new(640, 360, "Mapgen demo");
    let tiles = Image::load("assets/tile.png", 1).unwrap();
    let sprites = Sprite::new_alpha_set(
        &Vec2::new(32, 32),
        &Vec2::new(tiles.width as int, tiles.height as int),
        tiles.pixels,
        &Vec2::new(-16, -16));
    let idx = app.add_sprite(~sprites[0].clone());
    for i in range(1,16) {
        app.add_sprite(~sprites[i].clone());
    }
    let FLOOR = idx;
    let CUBE = idx + 1;
    let XWALL = idx + 2;
    let YWALL = idx + 3;
    let XYWALL = idx + 4;
    let OWALL = idx + 5;
    let AVATAR = idx + 6;
    let WATER = idx + 7;
    let CURSOR_BOTTOM = idx + 8;
    let CURSOR_TOP = idx + 9;

    let mut area = Area::new();
    /*
    static TERRAIN: &'static str = "
################################
#~~..###########################
#~..........#############.....##
#....##.###.####....##....###.##
#....##.###......##.##.##..#..##
#######~########.##....##.###.##
#######~########....#####.....##
###........######.#######.######
###..####..##~~~........#.######
##...####...#~~~........#.######
##...####...#~~~.............###
###..####..##.~.........##.#####
###.......###...........##..####
######.#########.#########.#####
######...........#########.#####
################################";
    area.gen_prefab(TERRAIN);
    */
    let mut rng = rand::rng();
    area.gen_cave(&mut rng);

    let test_map = dijkstra::build_map(
        ~[Location(Point2::new(0i8, 0i8))], |n| area.walk_neighbors(n), 666);
    for y in range(-16, 17) {
        for x in range(-16, 17) {
            let p = Location(Point2::new(x as i8, y as i8));
            match test_map.find(&p) {
                Some(&n) => print!("{:3u} ", n),
                _ => print!("{:3u} ", 999u),
            };
        }
        println!("");
    }

    while app.alive {
        app.set_color(&Vec4::new(0.0f32, 0.1f32, 0.2f32, 1f32));
        app.fill_rect(&RectUtil::new(0.0f32, 0.0f32, 640.0f32, 360.0f32));
        app.set_color(&Vec4::new(0.1f32, 0.3f32, 0.6f32, 1f32));
        // XXX: Horrible prototype code, figure out cleaning.

        let origin = Vec2::new(320.0f32, 180.0f32);

        // Mouse cursoring
        let mouse = app.get_mouse();
        let cursor_chart_pos = screen_to_chart(&mouse.pos.add_v(&origin.neg()).add_v(&Vec2::new(8.0f32, 0.0f32)));

        if app.screen_area().contains(&mouse.pos) {
            if mouse.left {
                area.dig(&Location(cursor_chart_pos));
            }

            if mouse.right {
                area.fill(&Location(cursor_chart_pos));
            }
        }

        let mut rect = Aabb2::new(
            screen_to_chart(&Point2::new(0f32, 0f32).add_v(&origin.neg())),
            screen_to_chart(&Point2::new(640f32, 392f32).add_v(&origin.neg())));
        rect = rect.grow(&screen_to_chart(&Point2::new(640f32, 0f32).add_v(&origin.neg())));
        rect = rect.grow(&screen_to_chart(&Point2::new(0f32, 392f32).add_v(&origin.neg())));

        // Draw floors
        for pt in rect.points() {
            let p = Location(pt);
            let offset = chart_to_screen(&pt).add_v(&origin);
            if area.get(&p) == area::Water {
                app.set_color(&Vec4::new(0.0f32, 0.5f32, 1.0f32, 1f32));
                app.draw_sprite(WATER, &offset);
            } else {
                app.set_color(&Vec4::new(0.7f32, 0.7f32, 0.8f32, 1f32));
                app.draw_sprite(FLOOR, &offset);
            }
        }

        // Draw cursor back under the protruding geometry.
        app.set_color(&Vec4::new(1.0f32, 0.4f32, 0.4f32, 1f32));
        app.draw_sprite(CURSOR_BOTTOM, &chart_to_screen(&cursor_chart_pos).add_v(&origin));

        // Draw walls
        for pt in rect.points() {
            let p = Location(pt);
            let offset = chart_to_screen(&pt).add_v(&origin);
            app.set_color(&Vec4::new(0.6f32, 0.5f32, 0.1f32, 1f32));
            if area.get(&p) == area::Wall {
                let left = is_solid(area.get(&(p + Vec2::new(-1, 0))));
                let rear = is_solid(area.get(&(p + Vec2::new(-1, -1))));
                let right = is_solid(area.get(&(p + Vec2::new(0, -1))));

                if left && right && rear {
                    app.draw_sprite(CUBE, &offset);
                    if !is_solid(area.get(&(p + Vec2::new(1, -1)))) ||
                       !is_solid(area.get(&(p + Vec2::new(1, 0)))) {
                        app.draw_sprite(YWALL, &offset);
                    }
                    if !is_solid(area.get(&(p + Vec2::new(-1, 1)))) ||
                       !is_solid(area.get(&(p + Vec2::new(0, 1)))) {
                        app.draw_sprite(XWALL, &offset);
                    }
                    if !is_solid(area.get(&(p + Vec2::new(1, 1)))) {
                        app.draw_sprite(OWALL, &offset);
                    }
                } else if left && right {
                    app.draw_sprite(XYWALL, &offset);
                } else if left {
                    app.draw_sprite(XWALL, &offset);
                } else if right {
                    app.draw_sprite(YWALL, &offset);
                } else {
                    app.draw_sprite(OWALL, &offset);
                };
            }

            if p == Location(Point2::new(0i8, 0i8)) {
                app.set_color(&Vec4::new(0.9f32, 0.9f32, 1.0f32, 1f32));
                app.draw_sprite(AVATAR, &offset);
            }
        }

        app.set_color(&Vec4::new(1.0f32, 0.4f32, 0.4f32, 1f32));
        app.draw_sprite(CURSOR_TOP, &chart_to_screen(&cursor_chart_pos).add_v(&origin));

        app.flush();

        for key in app.key_buffer().iter() {
            if key.code == key::ESC {
                return;
            }
        }
    }
}

pub fn chart_to_screen(map_pos: &Point2<i8>) -> Point2<f32> {
    Point2::new(
        16.0 * (map_pos.x as f32) - 16.0 * (map_pos.y as f32),
        8.0 * (map_pos.x as f32) + 8.0 * (map_pos.y as f32))
}

pub fn screen_to_chart(screen_pos: &Point2<f32>) -> Point2<i8> {
    let column = (screen_pos.x / 16.0).floor();
    let row = ((screen_pos.y - column * 8.0) / 16.0).floor();
    Point2::new((column + row) as i8, row as i8)
}
