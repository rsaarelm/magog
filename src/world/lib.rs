#[crate_id = "world"];
#[desc = "Game world logic"];
#[crate_type = "rlib"];
#[feature(globs)];

extern crate collections;
extern crate num;
extern crate time;
extern crate rand;
extern crate cgmath;
extern crate color;
extern crate calx;

pub mod dijkstra;
pub mod area;
pub mod fov;
pub mod areaview;
pub mod mapgen;
pub mod transform;
pub mod sprite;
pub mod mob;
pub mod state;
