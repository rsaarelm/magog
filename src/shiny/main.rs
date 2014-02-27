extern crate cgmath;
extern crate color;
extern crate glutil;
extern crate calx;

use std::cmp::max;
use glutil::app::App;
use glutil::app;
use cgmath::aabb::{Aabb, Aabb2};
use cgmath::point::{Point, Point2};
use cgmath::vector::{Vec2};
use color::rgb::{ToRGB, consts};
use calx::rectutil::RectUtil;

static WHAT_DO_YOU_THINK_NIGHTMARE_DOG: &'static str =
"This is of no consequence, the new Tower of Babel nears completion. All \
humanity will be made whole again, a nightmare god of flesh and bone, singing \
one chorus with its million mouths.";

static MIN_SPACE: uint = 5;


fn print_words<C: ToRGB>(app: &mut App, area: &Aabb2<f32>, color: &C, text: &str) {
    let words: ~[&str] = text.split(' ').collect();
    let bounds = words.map(|&w| app.string_bounds(w).dim().x as uint);
    let mut i = 0;
    let origin = area.min().add_v(&Vec2::new(0.0, app::FONT_HEIGHT)).to_vec();
    let width = area.dim().x;
    let max_lines = (area.dim().y / app::FONT_HEIGHT) as uint;
    let mut pos = origin;
    let mut line = 0;
    while i < words.len() && line < max_lines {
        let n = max(1, num_fitting_words(width as uint, MIN_SPACE, bounds.slice(i, bounds.len())));
        for j in range(i, i + n) {
            app.draw_string(&pos, color, words[j]);
            pos.x += bounds[j] as f32 + MIN_SPACE as f32;
        }
        i += n;
        pos.x = origin.x;
        pos.y += app::FONT_HEIGHT;
        line += 1;
    }

    fn num_fitting_words(span: uint, space: uint, lengths: &[uint]) -> uint {
        if lengths.len() == 0 { return 0 }
        let mut total = lengths[0];
        for i in range(1, lengths.len()) {
            let new_total = total + space + lengths[i];
            if new_total > span {
                return i;
            }
            total = new_total;
        }
        return lengths.len();
    }
}

pub fn main() {
    let mut app = App::new(640, 360, "Shiny!");

    while app.alive {
        app.fill_rect(&RectUtil::new(0.0f32, 0.0f32, 640.0f32, 360.0f32), &consts::MIDNIGHTBLUE);
        let area : Aabb2<f32> = RectUtil::new(0.0f32, 0.0f32, 213.0f32, 120.0f32);
        for p in area.points() {
            app.fill_rect(&Aabb2::new(
                    p.mul_s(3f32),
                    p.mul_s(3f32).add_v(&Vec2::new(2f32, 2f32))),
                    &consts::DARKSLATEGRAY);
        }

        let text_zone = Aabb2::new(Point2::new(4.0f32, 200.0f32), Point2::new(240.0f32, 360.0f32));
        print_words(&mut app, &text_zone.add_v(&Vec2::new(1.0f32, 1.0f32)), &consts::BLACK,
            WHAT_DO_YOU_THINK_NIGHTMARE_DOG);
        print_words(&mut app, &text_zone, &consts::SALMON,
            WHAT_DO_YOU_THINK_NIGHTMARE_DOG);

        app.flush();
    }
}
