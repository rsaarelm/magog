extern mod cgmath;
extern mod glutil;
extern mod calx;
extern mod stb;

use std::hashmap::HashMap;

use glutil::app::App;
use glutil::atlas::Sprite;
use cgmath::aabb::{Aabb2};
use cgmath::point::{Point, Point2};
use cgmath::vector::{Vec2, Vec4};
use calx::rectutil::RectUtil;
use stb::image::Image;
use calx::text::Map2DUtil;

#[deriving(Eq)]
enum TerrainType {
    Wall,
    Floor,
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
#....###########################
#...........#############.....##
#....##.###.####....##....###.##
#....##.###......##.##.##..#..##
#######.########.##....##.###.##
#######.########....#####.....##
###........######.#######.######
###..####..##...........#.######
##...####...#...........#.######
##...####...#................###
###..####..##...........##.#####
###.......###...........##..####
######.#########.#########.#####
######...........#########.#####
################################";
        for (c, x, y) in TERRAIN.chars().map2d() {
            if c == '.' {
                ret.set.insert(Point2::new(x as i8, y as i8), Floor);
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
    app.add_sprite(~sprites[1].clone());
    app.add_sprite(~sprites[2].clone());
    app.add_sprite(~sprites[3].clone());
    app.add_sprite(~sprites[4].clone());
    app.add_sprite(~sprites[5].clone());
    app.add_sprite(~sprites[6].clone());
    let area = Area::new();
    while app.alive {
        app.set_color(&Vec4::new(0.0f32, 0.1f32, 0.2f32, 1f32));
        app.fill_rect(&RectUtil::new(0.0f32, 0.0f32, 640.0f32, 360.0f32));
        app.set_color(&Vec4::new(0.1f32, 0.3f32, 0.6f32, 1f32));
        let rect : Aabb2<i8> = RectUtil::new(0i8, 0i8, 16i8, 16i8);
        for p in rect.points() {
            let offset = Vec2::new(
                320.0 + 16.0 * (p.x as f32) - 16.0 * (p.y as f32),
                24.0 + 8.0 * (p.x as f32) + 8.0 * (p.y as f32));
            app.set_color(&Vec4::new(0.7f32, 0.7f32, 0.8f32, 1f32));
            // Floor
            app.draw_sprite(idx, &offset);
            if area.get(&p) == Wall {
                // XXX: Horrible prototype code, figure out cleaning.
                app.set_color(&Vec4::new(0.6f32, 0.5f32, 0.1f32, 1f32));
                let left = area.get(&p.add_v(&Vec2::new(-1i8, 0i8))) == Wall;
                let rear = area.get(&p.add_v(&Vec2::new(-1i8, -1i8))) == Wall;
                let right = area.get(&p.add_v(&Vec2::new(0i8, -1i8))) == Wall;
                if left && right && rear {
                    app.draw_sprite(idx + 1, &offset);
                    if area.get(&p.add_v(&Vec2::new(1i8, -1i8))) != Wall ||
                       area.get(&p.add_v(&Vec2::new(1i8, 0i8))) != Wall {
                        app.draw_sprite(idx + 3, &offset);
                    }
                    if area.get(&p.add_v(&Vec2::new(-1i8, 1i8))) != Wall ||
                       area.get(&p.add_v(&Vec2::new(0i8, 1i8))) != Wall {
                        app.draw_sprite(idx + 2, &offset);
                    }
                    if area.get(&p.add_v(&Vec2::new(1i8, 1i8))) != Wall {
                        app.draw_sprite(idx + 5, &offset);
                    }
                } else if left && right {
                    app.draw_sprite(idx + 4, &offset);
                } else if left {
                    app.draw_sprite(idx + 2, &offset);
                } else if right {
                    app.draw_sprite(idx + 3, &offset);
                } else {
                    app.draw_sprite(idx + 5, &offset);
                };
            }

            if p == Point2::new(8i8, 8i8) {
                app.set_color(&Vec4::new(0.9f32, 0.9f32, 1.0f32, 1f32));
                app.draw_sprite(idx + 6, &offset);
            }
        }

        app.flush();
    }
}
