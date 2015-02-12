/*!
Miscellaneous utilities grab-bag.

 */

#![crate_name="calx_util"]
#![feature(core, collections, hash, io, std_misc)]
#![feature(plugin)]
#![plugin(regex_macros)]

extern crate collections;
extern crate "rustc-serialize" as rustc_serialize;
extern crate time;
extern crate rand;
extern crate regex_macros;
extern crate regex;
extern crate image;

pub use rgb::{Rgb, Rgba};
pub use geom::{V2, V3, Rect, RectIter};
pub use img::{color_key};
pub use atlas::{AtlasBuilder, Atlas, AtlasItem};
pub use dijkstra::{DijkstraNode, Dijkstra};
pub use encode_rng::{EncodeRng};

mod atlas;
mod dijkstra;
mod geom;
mod encode_rng;
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

/// Clamp a value to range.
pub fn clamp<C: PartialOrd+Copy>(mn: C, mx: C, x: C) -> C {
    if x < mn { mn }
    else if x > mx { mx }
    else { x }
}

/// Deterministic noise.
pub fn noise(n: i32) -> f32 {
    let n = (n << 13) ^ n;
    let m = (n * (n * n * 15731 + 789221) + 1376312589) & 0x7fffffff;
    1.0 - m as f32 / 1073741824.0
}
