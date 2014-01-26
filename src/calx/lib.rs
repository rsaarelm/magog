#[desc = "Shared gamelib"];
#[license = "GPLv3"];
#[feature(globs)];

extern mod sdl2;
extern mod extra;

pub mod text;
pub mod app;

pub fn hello() {
    println!("Hello, world!");
}
