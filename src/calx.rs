#![crate_id="calx#0.1"]
#![feature(globs)]
#![feature(macro_rules)]

extern crate cgmath;
extern crate collections;
extern crate color;
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
    pub mod dijkstra;
    pub mod area;
    pub mod fov;
    pub mod areaview;
    pub mod mapgen;
    pub mod transform;
    pub mod sprite;
    pub mod mob;
    pub mod state;
}

/*
pub mod game {
    pub mod game;
    pub mod main;
}

pub mod mapedit {
    pub mod main;
}
*/

pub mod demo {
    pub mod main;
}

pub mod enginetest {
    pub mod main;
}

pub fn main() {
    let cmd = if os::args().len() > 1 { os::args().get(1).to_owned() } else { "game".to_owned() };
    match cmd.as_slice() {
        /*
        "mapedit" => mapedit::main::main(),
        "game" => game::main::main(),
        */
        "demo" => demo::main::main(),
        "enginetest" => enginetest::main::main(),
        _ => println!("Unknown command")
    }
}
