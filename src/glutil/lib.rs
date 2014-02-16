#[crate_id = "glutil"];
#[desc = "OpenGL rendering utilities"];
#[license = "MIT"];
#[crate_type = "rlib"];

#[feature(macro_rules)];

extern crate opengles;
extern crate calx;
extern crate stb;
extern crate cgmath;
extern crate glfw = "glfw-rs";

#[macro_escape]
pub mod gl_check;

pub mod shader;
pub mod texture;
pub mod app;
pub mod atlas;
pub mod recter;
pub mod key;
pub mod buffer;
