#![crate_name="calx"]
#![feature(globs)]
#![feature(macro_rules)]

extern crate collections;
extern crate hgl;
extern crate gl;
extern crate glfw;
extern crate rand;
extern crate cgmath;
extern crate libc;
extern crate time;
extern crate serialize;
extern crate uuid;

pub mod asciimap;
pub mod color;
pub mod debug;
pub mod engine;
#[macro_escape]
pub mod gen_id;
pub mod pack_rect;
pub mod rectutil;
pub mod text;
pub mod tile;
pub mod timing;
pub mod world;

pub mod stb {
    pub mod image;
}

