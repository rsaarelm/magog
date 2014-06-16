#![crate_id="calx#0.1"]
#![feature(globs)]
#![feature(macro_rules)]

extern crate cgmath;
extern crate collections;
extern crate gl;
extern crate glfw;
extern crate hgl;
extern crate libc;
extern crate num;
extern crate rand;
extern crate serialize;
extern crate time;

use std::os;

pub mod asciimap;
pub mod color;
pub mod engine;
pub mod gen_id;
pub mod pack_rect;
pub mod rectutil;
pub mod text;
pub mod tile;
pub mod timing;

pub mod stb {
    pub mod image;
    pub mod truetype;
}

pub mod world {
    pub mod ai;
    pub mod area;
    pub mod dijkstra;
    pub mod fov;
    pub mod geomorph;
    pub mod mapgen;
    pub mod mobs;
    pub mod spawn;
    pub mod terrain;
    pub mod world;
}

pub mod view {
    pub mod worldview;
}

pub mod game {
    pub mod main;
}

pub fn main() {
    let cmd = if os::args().len() > 1 { os::args().get(1).to_string() } else { "game".to_string() };
    match cmd.as_slice() {
        "game" => game::main::main(),
        _ => println!("Unknown command")
    }
}
