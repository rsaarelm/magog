#![crate_name="blot"]
#![feature(phase)]

extern crate time;
extern crate sync;
extern crate collections;

extern crate glfw;
extern crate gfx;
#[phase(plugin)]
extern crate gfx_macros;
extern crate image;

pub use canvas::Canvas;
pub use canvas::{Render, Input};

pub mod canvas;
