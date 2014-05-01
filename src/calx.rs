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
pub mod text;
pub mod pack_rect;
pub mod rectutil;
pub mod gen_id;
pub mod app;
pub mod renderer;
pub mod tile;
pub mod key;
pub mod timing;

pub mod glutil {
    pub mod glrenderer;
    pub mod atlas;
    pub mod recter;
    pub mod framebuffer;
}

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

pub mod demogame {
    pub mod game;
    pub mod main;
}

pub mod mapedit {
    pub mod main;
}

pub fn main() {
    let cmd = if os::args().len() > 1 { os::args()[1] } else { ~"demogame" };
    match cmd.as_slice() {
        "mapedit" => mapedit::main::main(),
        "demogame" => demogame::main::main(),
        _ => println!("Unknown command")
    }
}
