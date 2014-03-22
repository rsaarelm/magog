#[crate_id = "world"];
#[desc = "Game world logic"];
#[crate_type = "rlib"];

extern crate collections;
extern crate cgmath;
extern crate num;

pub mod dijkstra;
pub mod area;
pub mod fov;
