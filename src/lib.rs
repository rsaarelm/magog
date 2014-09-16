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
pub use canvas::{Render, Text, KeyPressed, KeyReleased};

pub mod canvas;
pub mod util;
pub mod key;
pub mod atlas;
