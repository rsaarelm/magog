#[crate_id = "glutil"];
#[desc = "OpenGL rendering utilities"];
#[license = "MIT"];
#[crate_type = "rlib"];

#[feature(macro_rules)];

extern crate gl;
extern crate hgl;
extern crate cgmath;
extern crate glfw = "glfw-rs";
extern crate color;
extern crate calx;
extern crate stb;

pub mod glrenderer;
pub mod atlas;
pub mod recter;
pub mod framebuffer;
