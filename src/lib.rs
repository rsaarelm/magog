/*!
Miscellaneous utilities grab-bag.

 */

#![crate_name="calx"]
#![feature(core, collections, std_misc, thread_sleep)]
#![feature(plugin, custom_attribute)]
#![plugin(regex_macros, glium_macros)]

extern crate collections;
extern crate rustc_serialize;
extern crate time;
extern crate rand;
extern crate regex;
extern crate image;
extern crate glutin;
#[macro_use]
extern crate glium;

use std::env;
use std::path::{Path, PathBuf};
use std::num::wrapping::Wrapping;

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

pub mod backend;
pub mod color;
pub mod text;
pub mod timing;
pub mod vorud;

pub trait Color: Sized {
    fn to_rgba(&self) -> [f32; 4];
    fn from_color<C: Color>(color: &C) -> Self;

    fn from_rgba(rgba: [f32; 4]) -> Self {
        Color::from_color(&rgb::Rgba::new(
                (rgba[0] * 255.0) as u8,
                (rgba[1] * 255.0) as u8,
                (rgba[2] * 255.0) as u8,
                (rgba[3] * 255.0) as u8))
    }
}

/// Clamp a value to range.
pub fn clamp<C: PartialOrd+Copy>(mn: C, mx: C, x: C) -> C {
    if x < mn { mn }
    else if x > mx { mx }
    else { x }
}

/// Deterministic noise.
pub fn noise(n: i32) -> f32 {
    let n = Wrapping(n);
    let n = (n << 13) ^ n;
    let m = (n * (n * n * Wrapping(15731) + Wrapping(789221)) + Wrapping(1376312589))
        & Wrapping(0x7fffffff);
    let Wrapping(m) = m;
    1.0 - m as f32 / 1073741824.0
}

/// Rectangle anchoring points.
#[derive(Copy, PartialEq, Debug)]
pub enum Anchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Top,
    Left,
    Right,
    Bottom,
    Center
}

/// Return the application data directory path for the current platform.
#[cfg(target_os = "macos")]
pub fn app_data_path(app_name: &str) -> PathBuf {
    Path::new(
        format!("{}/Library/Application Support/{}",
                env::var("HOME").unwrap(), app_name))
    .to_path_buf()
}

#[cfg(target_os = "windows")]
pub fn app_data_path(app_name: &str) -> PathBuf {
    Path::new(
    format!("{}\\{}", env::var("APPDATA").unwrap(), app_name))
    .to_path_buf()
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
pub fn app_data_path(app_name: &str) -> PathBuf {
    Path::new(
        &format!("{}/.config/{}", env::var("HOME").unwrap(), app_name))
    .to_path_buf()
}

#[cfg(test)]
mod test {
    #[test]
    fn test_noise() {
        use super::noise;

        for i in 0i32..100 {
            assert!(noise(i) >= -1.0 && noise(i) <= 1.0);
        }
    }
}
