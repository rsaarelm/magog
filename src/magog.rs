#![crate_id="magog#0.0.1"]
#![feature(globs)]
#![feature(macro_rules)]

extern crate calx;
extern crate cgmath;
extern crate num;
extern crate rand;
extern crate time;

pub mod world {
    //pub mod ai;
    pub mod area;
    pub mod dijkstra;
    pub mod fov;
    pub mod geomorph;
    pub mod mapgen;
    pub mod mobs;
    //pub mod spawn;
    pub mod terrain;
    pub mod system;
}

pub mod view {
    pub mod worldview;
    pub mod tilecache;
}

pub mod game {
    pub mod main;
}

pub fn main() {
    game::main::main()
}
