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
use area::{DIRECTIONS, Location};
use areaview::AreaView;

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
    println!("{}", idx);
    for i in range(1,16) {
        app.add_sprite(~sprites[i].clone());
    }
    let mut area = Area::new();
    let area_view = AreaView::new();
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

        let origin = Vec2::new(320.0f32, 180.0f32);
        let mouse = app.get_mouse();
        let cursor_chart_pos = areaview::screen_to_chart(
            &mouse.pos.add_v(&origin.neg()).add_v(&Vec2::new(8.0f32, 0.0f32)));

        if app.screen_area().contains(&mouse.pos) {
            if mouse.left {
                area.dig(&Location(cursor_chart_pos));
            }

            if mouse.right {
                area.fill(&Location(cursor_chart_pos));
            }
        }

        area_view.draw(&area, &mut app);

        app.flush();

        for key in app.key_buffer().iter() {
            if key.code == key::ESC {
                return;
            }
        }
    }
}

