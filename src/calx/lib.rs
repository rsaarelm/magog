#[crate_id = "calx"];
#[desc = "Shared gamelib"];
#[license = "MIT"];
#[feature(globs)];
#[crate_type = "rlib"];

extern crate collections;
extern crate cgmath;

pub mod text;
pub mod pack_rect;
pub mod rectutil;
