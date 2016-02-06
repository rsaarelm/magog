/*!
Miscellaneous utilities grab-bag.

 */

#![crate_name="calx"]

#![feature(deprecated)]
#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate serde;
extern crate time;
extern crate rand;
extern crate num;
extern crate vec_map;
extern crate nalgebra;
extern crate image;
extern crate bincode;
extern crate calx_alg;

#[macro_use] extern crate glium;

use std::path::{Path, PathBuf};

pub use geom::{V2, V3, Rect, RectIter, IterTiles};

#[deprecated] mod geom;
pub mod backend;
pub mod timing;

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


#[macro_export]
macro_rules! count_exprs {
    () => { 0 };
    ($e:expr) => { 1 };
    ($e:expr, $($es:expr),+) => { 1 + count_exprs!($($es),*) };
}
