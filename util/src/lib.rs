/*!
Miscellaneous utilities grab-bag.

 */

#![crate_name="calx_util"]
#![allow(unstable)]

extern crate collections;
extern crate "rustc-serialize" as rustc_serialize;
extern crate time;
extern crate image;

pub use rgb::{Rgb, Rgba};
pub use geom::{Rect, V2, RectIter};
pub use img::{color_key};
pub use atlas::{AtlasBuilder, Atlas};
pub use dijkstra::{DijkstraNode, Dijkstra};

mod atlas;
mod dijkstra;
mod geom;
mod img;
mod primitive;
mod rgb;

pub mod color;
pub mod text;
pub mod timing;
pub mod vorud;

pub trait Color {
    fn to_rgba(&self) -> [f32; 4];
}
