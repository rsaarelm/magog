#![crate_name="calx"]
#![feature(phase)]
#![feature(tuple_indexing)]
#![feature(if_let)]

extern crate time;
extern crate sync;
extern crate collections;
extern crate serialize;

extern crate glfw;
extern crate gfx;
#[phase(plugin)]
extern crate gfx_macros;
extern crate image;

pub use canvas::{Canvas, Context};
pub use canvas::{Image};
pub use rgb::{Rgb};
pub use geom::{Rect, V2};
pub use util::{color_key};
pub use fonter::{Fonter, CanvasWriter};

mod atlas;
mod canvas;
mod fonter;
mod geom;
mod glfw_key;
mod rgb;
mod util;
pub mod color;
pub mod dijkstra;
pub mod event;
pub mod key;
pub mod text;
pub mod timing;
pub mod vorud;
