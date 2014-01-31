#[crate_id = "calx"];
#[desc = "Shared gamelib"];
#[license = "MIT"];
#[feature(globs)];
#[crate_type = "rlib"];

extern mod extra;
extern mod cgmath;

pub mod text;
pub mod pack_rect;
