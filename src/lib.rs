#![crate_name="calx"]
#![feature(phase)]
#![feature(tuple_indexing)]

extern crate time;
extern crate sync;
extern crate collections;

extern crate glfw;
extern crate gfx;
#[phase(plugin)]
extern crate gfx_macros;
extern crate image;

pub use canvas::{Canvas, Context};
pub use canvas::{Render, Text, KeyPressed, KeyReleased};
pub use color::{Rgb};

pub mod atlas;
pub mod canvas;
pub mod color;
pub mod geom;
pub mod key;
pub mod text;
pub mod timing;
pub mod util;
