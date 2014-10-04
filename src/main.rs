#![crate_name="magog"]
#![feature(globs)]
#![feature(macro_rules)]
#![feature(tuple_indexing)]
#![comment = "Magog toplevel and display interface"]

extern crate image;
extern crate calx;
extern crate world;
extern crate time;

use calx::{V2};
use calx::color;

pub mod tilecache;

pub fn main() {
    let mut canvas = calx::Canvas::new();
    tilecache::init(&mut canvas);

    for evt in canvas.run() {
        match evt {
            calx::Render(ctx) => {
                ctx.clear(&color::BLACK);
                ctx.draw_image(V2(32, 32), 0.4, tilecache::get(tilecache::tile::AVATAR), &color::ORANGE);
            }
            calx::KeyPressed(calx::key::KeyEscape) => {
                return;
            }
            _ => ()
        }
    }
}
