extern mod cgmath;
extern mod glutil;
extern mod calx;
extern mod stb;

use std::hashmap::HashMap;

use glutil::app::App;
use glutil::key;
use glutil::atlas::Sprite;
use cgmath::aabb::{Aabb, Aabb2};
use cgmath::point::{Point, Point2};
use cgmath::vector::{Vec2, Vec4};
use calx::rectutil::RectUtil;
use stb::image::Image;
use calx::text::Map2DUtil;

#[deriving(Eq)]
enum TerrainType {
    Wall,
    Floor,
    Water,
}

pub fn solid(t: TerrainType) -> bool {
    t == Wall
}

struct Area {
    set: HashMap<Point2<i8>, TerrainType>,
}

impl Area {
    pub fn new() -> Area {
        let mut ret = Area {
            set: HashMap::new(),
        };
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
        for (c, x, y) in TERRAIN.chars().map2d() {
            if c == '.' {
                ret.set.insert(Point2::new(x as i8, y as i8), Floor);
            }
            if c == '~' {
                ret.set.insert(Point2::new(x as i8, y as i8), Water);
            }
        }
        ret
    }

    pub fn get(&self, p: &Point2<i8>) -> TerrainType {
        match self.set.find(p) {
            None => Wall,
            Some(&t) => t
        }
    }

    pub fn dig(&mut self, p: &Point2<i8>) {
        self.set.insert(*p, Floor);
    }

    pub fn fill(&mut self, p: &Point2<i8>) {
        self.set.insert(*p, Wall);
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
    while app.alive {
        app.set_color(&Vec4::new(0.0f32, 0.1f32, 0.2f32, 1f32));
        app.fill_rect(&RectUtil::new(0.0f32, 0.0f32, 640.0f32, 360.0f32));
        app.set_color(&Vec4::new(0.1f32, 0.3f32, 0.6f32, 1f32));
        // XXX: Horrible prototype code, figure out cleaning.

        let origin = Vec2::new(320.0f32, 24.0f32);

        // Mouse cursoring
        let mouse = app.get_mouse();
        let cursor_chart_pos = screen_to_chart(&mouse.pos.add_v(&origin.neg()).add_v(&Vec2::new(8.0f32, 0.0f32)));

        if app.screen_area().contains(&mouse.pos) {
            if mouse.left {
                area.dig(&cursor_chart_pos);
            }

            if mouse.right {
                area.fill(&cursor_chart_pos);
            }
        }

        let mut rect = Aabb2::new(
            screen_to_chart(&Point2::new(0f32, 0f32).add_v(&origin.neg())),
            screen_to_chart(&Point2::new(640f32, 360f32).add_v(&origin.neg())));
        rect = rect.grow(&screen_to_chart(&Point2::new(640f32, 0f32).add_v(&origin.neg())));
        rect = rect.grow(&screen_to_chart(&Point2::new(0f32, 360f32).add_v(&origin.neg())));

        // Draw floors
        for p in rect.points() {
            let offset = chart_to_screen(&p).add_v(&origin);
            if area.get(&p) == Water {
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
        for p in rect.points() {
            let offset = chart_to_screen(&p).add_v(&origin);
            app.set_color(&Vec4::new(0.6f32, 0.5f32, 0.1f32, 1f32));
            if area.get(&p) == Wall {
                let left = solid(area.get(&p.add_v(&Vec2::new(-1i8, 0i8))));
                let rear = solid(area.get(&p.add_v(&Vec2::new(-1i8, -1i8))));
                let right = solid(area.get(&p.add_v(&Vec2::new(0i8, -1i8))));

                if left && right && rear {
                    app.draw_sprite(CUBE, &offset);
                    if !solid(area.get(&p.add_v(&Vec2::new(1i8, -1i8)))) ||
                       !solid(area.get(&p.add_v(&Vec2::new(1i8, 0i8)))) {
                        app.draw_sprite(YWALL, &offset);
                    }
                    if !solid(area.get(&p.add_v(&Vec2::new(-1i8, 1i8)))) ||
                       !solid(area.get(&p.add_v(&Vec2::new(0i8, 1i8)))) {
                        app.draw_sprite(XWALL, &offset);
                    }
                    if !solid(area.get(&p.add_v(&Vec2::new(1i8, 1i8)))) {
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

            if p == Point2::new(8i8, 8i8) {
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
