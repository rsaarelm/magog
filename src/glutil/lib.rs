#[crate_id = "glutil"];
#[desc = "OpenGL rendering utilities"];
#[license = "MIT"];
#[crate_type = "rlib"];

#[feature(macro_rules)];

extern mod opengles;
extern mod calx;
extern mod stb;
extern mod cgmath;
extern mod glfw = "glfw-rs";

#[macro_escape]
pub mod gl_check;

pub mod shader;
pub mod texture;
pub mod app;
pub mod atlas;
pub mod recter;
pub mod key;
pub mod buffer;
