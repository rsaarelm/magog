#![crate_name="util"]
#![allow(unstable)]

extern crate collections;
extern crate "rustc-serialize" as rustc_serialize;
extern crate time;
extern crate image;

pub use rgb::{Rgb, Rgba};
// pub use geom::{Rect, V2, RectIter};
// pub use util::{color_key};
// pub use util::{Primitive};

// mod atlas;
// mod geom;
mod rgb;
// mod util;

pub mod color;
// pub mod dijkstra;
pub mod text;
pub mod timing;
pub mod vorud;

pub trait Color {
    fn to_rgba(&self) -> [f32; 4];
}
