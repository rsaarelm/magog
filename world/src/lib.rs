#![crate_name="world"]
#![feature(globs)]
#![feature(macro_rules)]
#![feature(tuple_indexing)]
#![comment = "Display independent world logic for Magog"]

extern crate num;
extern crate rand;
extern crate time;
extern crate cgmath;
extern crate image;
extern crate calx;
extern crate uuid;

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
