#![crate_name="magog"]
#![feature(globs)]
#![feature(macro_rules)]

extern crate num;
extern crate rand;
extern crate time;
extern crate cgmath;
extern crate image;
extern crate calx;
extern crate uuid;

pub mod world {
    pub mod area;
    pub mod dijkstra;
    pub mod fov;
    pub mod mapgen;
    pub mod mobs;
    pub mod spatial;
    pub mod spawn;
    pub mod system;
    pub mod terrain;
    pub mod world;
}

pub mod view {
    pub mod drawable;
    //pub mod worldview;
    pub mod tilecache;
    pub mod main;
    //pub mod titlestate;
    //pub mod gamestate;
}

pub fn main() {
    view::main::main()
}
