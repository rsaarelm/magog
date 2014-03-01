use cgmath::point::{Point, Point2};
use cgmath::vector::{Vec2};
use cgmath::aabb::{Aabb, Aabb2};
use color::rgb::RGB;
use color::rgb::consts::*;
use calx::rectutil::RectUtil;
use area;
use area::{Location, Area, is_solid};
use fov::Fov;
use glutil::app::{App, SPRITE_INDEX_START};

pub static FLOOR : uint = SPRITE_INDEX_START + 10;
pub static CUBE : uint = SPRITE_INDEX_START + 1;
pub static XWALL : uint = SPRITE_INDEX_START + 16;
pub static YWALL : uint = SPRITE_INDEX_START + 17;
pub static XYWALL : uint = SPRITE_INDEX_START + 18;
pub static OWALL : uint = SPRITE_INDEX_START + 19;
pub static AVATAR : uint = SPRITE_INDEX_START + 26;
pub static WATER : uint = SPRITE_INDEX_START + 12;
pub static CURSOR_BOTTOM : uint = SPRITE_INDEX_START + 8;
pub static CURSOR_TOP : uint = SPRITE_INDEX_START + 9;
pub static DOWNSTAIRS : uint = SPRITE_INDEX_START + 14;

static REMEMBER_COL: &'static RGB<u8> = &DARKSLATEGRAY;
static UNSEEN_COL: &'static RGB<u8> = &MAROON;
static WATER_COL: &'static RGB<u8> = &MEDIUMSLATEBLUE;
static FLOOR_COL: &'static RGB<u8> = &SLATEGRAY;
static WALL_COL: &'static RGB<u8> = &DARKGOLDENROD;
static AVATAR_COL: &'static RGB<u8> = &AZURE;
static CURSOR_COL: &'static RGB<u8> = &FIREBRICK;

pub fn draw_area(
    area: &Area, app: &mut App, center: &Location,
    seen: &Fov, remembered: &Fov) {
    // XXX: Horrible prototype code, figure out cleaning.

    let origin = Vec2::new(320.0f32, 180.0f32);

    // Mouse cursoring
    let mouse = app.get_mouse();
    let cursor_chart_pos = screen_to_chart(&mouse.pos.add_v(&origin.neg()).add_v(&Vec2::new(8.0f32, 0.0f32)));

    let mut rect = Aabb2::new(
        screen_to_chart(&Point2::new(0f32, 0f32).add_v(&origin.neg())),
        screen_to_chart(&Point2::new(640f32, 392f32).add_v(&origin.neg())));
    rect = rect.grow(&screen_to_chart(&Point2::new(640f32, 0f32).add_v(&origin.neg())));
    rect = rect.grow(&screen_to_chart(&Point2::new(0f32, 392f32).add_v(&origin.neg())));

    let &Location(ref offset) = center;
    let pos_offset = Vec2::new(offset.x as int, offset.y as int);

    // Draw floors
    for pt in rect.points() {
        let p = Location(pt) + pos_offset;

        let offset = chart_to_screen(&pt).add_v(&origin);
        let mut color =
            if area.get(&p) == area::Water {
                WATER_COL
            } else {
                FLOOR_COL
            };

        /*
        if !seen.contains(&p) {
            if remembered.contains(&p) {
                color = REMEMBER_COL;
            } else {
                continue;
            }
        }
        */
        if !seen.contains(&p) {
            if !remembered.contains(&p) {
                color = REMEMBER_COL;
            }
        }

        if color == UNSEEN_COL { continue; } // Don't display debug stuff.
        if area.get(&p) == area::Water {
            app.draw_sprite(WATER, &offset, color);
        } else if area.get(&p) == area::Downstairs {
            app.draw_sprite(DOWNSTAIRS, &offset, color);
        } else {
            app.draw_sprite(FLOOR, &offset, color);
        }
    }

    // Draw cursor back under the protruding geometry.
    app.draw_sprite(CURSOR_BOTTOM, &chart_to_screen(&cursor_chart_pos).add_v(&origin), CURSOR_COL);

    // Draw walls
    for pt in rect.points() {
        let p = Location(pt) + pos_offset;
        let offset = chart_to_screen(&pt).add_v(&origin);
        let mut color = WALL_COL;

        /*
        if !seen.contains(&p) {
            if remembered.contains(&p) {
                color = REMEMBER_COL;
            } else {
                continue;
            }
        }
        */
        if !seen.contains(&p) {
            if !remembered.contains(&p) {
                color = REMEMBER_COL;
            }
        }

        if area.get(&p) == area::Wall {
            let left = is_solid(area.get(&(p + Vec2::new(-1, 0))));
            let rear = is_solid(area.get(&(p + Vec2::new(-1, -1))));
            let right = is_solid(area.get(&(p + Vec2::new(0, -1))));

            if left && right && rear {
                app.draw_sprite(CUBE, &offset, color);
                if !is_solid(area.get(&(p + Vec2::new(1, -1)))) ||
                   !is_solid(area.get(&(p + Vec2::new(1, 0)))) {
                    app.draw_sprite(YWALL, &offset, color);
                }
                if !is_solid(area.get(&(p + Vec2::new(-1, 1)))) ||
                   !is_solid(area.get(&(p + Vec2::new(0, 1)))) {
                    app.draw_sprite(XWALL, &offset, color);
                }
                if !is_solid(area.get(&(p + Vec2::new(1, 1)))) {
                    app.draw_sprite(OWALL, &offset, color);
                }
            } else if left && right {
                app.draw_sprite(XYWALL, &offset, color);
            } else if left {
                app.draw_sprite(XWALL, &offset, color);
            } else if right {
                app.draw_sprite(YWALL, &offset, color);
            } else {
                app.draw_sprite(OWALL, &offset, color);
            };
        }

        if &p == center {
            app.draw_sprite(AVATAR, &offset, AVATAR_COL);
        }
    }

    app.draw_sprite(CURSOR_TOP, &chart_to_screen(&cursor_chart_pos).add_v(&origin), CURSOR_COL);
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
