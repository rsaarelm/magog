#![crate_name="magog"]
#![feature(globs)]
#![feature(macro_rules)]
#![feature(tuple_indexing)]
#![comment = "Magog toplevel and display interface"]

extern crate image;
extern crate calx;
extern crate world;
extern crate time;

use calx::color;
use calx::event;
use calx::key;
use world::{Location};

mod drawable;
pub mod tilecache;
pub mod viewutil;
pub mod worldview;

pub fn main() {
    let mut canvas = calx::Canvas::new();
    tilecache::init(&mut canvas);

    let sav = world::save();
    println!("{}", sav);

    for evt in canvas.run() {
        match evt {
            event::Render(ctx) => {
                ctx.clear(&color::BLACK);
                let camera = Location::new(0, 0);
                worldview::draw_world(&camera, ctx);
            }
            event::KeyPressed(key::KeyEscape) => {
                return;
            }
            _ => ()
        }
    }
}
