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
pub use canvas::{Image};
pub use canvas::{Rgb};
pub use geom::{Rect, V2};
pub use util::{color_key};

mod atlas;
mod canvas;
mod geom;
mod util;
pub mod color;
pub mod event;
pub mod key;
pub mod text;
pub mod timing;
