#![crate_name="blot"]
#![feature(phase)]

extern crate sync;
extern crate glfw;
extern crate gfx;
#[phase(plugin)]
extern crate gfx_macros;
extern crate image;

pub mod window;
