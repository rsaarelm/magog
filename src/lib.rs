#![crate_name="blot"]
#![feature(phase)]

extern crate time;
extern crate sync;

extern crate glfw;
extern crate gfx;
#[phase(plugin)]
extern crate gfx_macros;
extern crate image;

pub use window::Window;
pub use window::{Render, Input};

pub mod window;
