#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate num;
extern crate rand;
extern crate bincode;
extern crate serde;
extern crate vec_map;
extern crate image;
extern crate euclid;
extern crate vitral;
extern crate vitral_atlas;
extern crate calx_alg;
extern crate calx_grid;
extern crate calx_color;
#[macro_use]
extern crate calx_ecs;
#[macro_use]
extern crate calx_resource;

use euclid::{Rect, Point2D, Size2D};

mod ability;

mod brush;
pub use brush::{Brush, BrushBuilder, Color, Frame, ImageRef, Splat};

mod command;
pub use command::Command;

mod components;
mod field;
mod flags;
mod fov;
mod item;

mod location;
pub use location::{Location, Portal};

mod location_set;
mod mutate;

mod query;
pub use query::Query;

mod spatial;
mod stats;

mod terraform;
pub use terraform::Terraform;

pub mod terrain;

mod world;
pub use world::World;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FovStatus {
    Seen,
    Remembered,
}

/// Return whether the given chart point is on the currently visible screen.
///
/// It is assumed that the chart point 0, 0 is at the center of the screen.
///
/// Since various bits of game logic are tied to the screen boundaries, the screen size is fixed as
/// a constant.
pub fn on_screen(chart_pos: Point2D<i32>) -> bool {
    const W: i32 = 39;
    const H: i32 = 22;
    let (x, y) = (chart_pos.x, chart_pos.y);

    x <= y + (W - 1) / 2 &&
    x >= y - (W - 1) / 2 &&
    x >= -H - y &&
    x <= H - 2 - y
}
