/*!
Miscellaneous utilities grab-bag.

 */

#![crate_name="calx"]
#![feature(core, collections, std_misc)]
#![feature(plugin, custom_attribute, unboxed_closures)]
#![feature(custom_derive)]
#![plugin(rand_macros)]

#[no_link] extern crate rand_macros;
extern crate collections;
extern crate rustc_serialize;
extern crate time;
extern crate rand;
extern crate num;
extern crate image;
extern crate glutin;
#[macro_use]
extern crate glium;

use std::env;
use std::path::{Path, PathBuf};
use std::num::Wrapping;

pub use rgb::{Rgb, Rgba};
pub use geom::{V2, V3, Rect, RectIter};
pub use img::{color_key};
pub use atlas::{AtlasBuilder, Atlas, AtlasItem};
pub use dijkstra::{DijkstraNode, Dijkstra};
pub use encode_rng::{EncodeRng};
pub use hex::{HexGeom, Dir6, HexFov};

mod atlas;
mod dijkstra;
mod geom;
mod encode_rng;
mod hex;
mod img;
mod primitive;
mod rgb;

pub mod backend;
pub mod color;
pub mod text;
pub mod timing;
pub mod vorud;

/// Things that describe a color.
pub trait ToColor {
    fn to_rgba(&self) -> [f32; 4];
}

/// Things that can be made from a color.
pub trait FromColor: Sized {
    fn from_rgba(rgba: [f32; 4]) -> Self;

    fn from_color<C: ToColor>(color: &C) -> Self {
        FromColor::from_rgba(color.to_rgba())
    }
}

pub trait Color: ToColor + FromColor {}

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
#[derive(Copy, Clone, PartialEq, Debug)]
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
pub fn app_data_path(_app_name: &str) -> PathBuf {
    use std::env;

    // Unless the Windows app was installed with an actual installer instead
    // of just being a portable .exe file, it shouldn't go around creating
    // strange directories but just use the local directory instead.

    // Path::new(
    // format!("{}\\{}", env::var("APPDATA").unwrap(), app_name))
    // .to_path_buf();

    match env::current_exe() {
        Ok(mut p) => { p.pop(); p }
        // If couldn't get self exe path, just use the local relative path and
        // hope for the best.
        _ => Path::new(".").to_path_buf()
    }
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
