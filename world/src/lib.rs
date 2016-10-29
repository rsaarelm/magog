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
    const SCREEN_W: f32 = 640.0;
    const SCREEN_H: f32 = 360.0;

    let screen_area = Rect::new(Point2D::new(0.0, 0.0), Size2D::new(SCREEN_W, SCREEN_H));
    let center = screen_area.origin + screen_area.size / 2.0;
    let screen_pos = chart_to_view(chart_pos) + center;
    let bounds = screen_area.inflate(-8.0, -8.0).translate(&Point2D::new(0.0, -4.0));
    return bounds.contains(&screen_pos);

    // XXX: Copy-pasted from display code.
    fn chart_to_view(chart_pos: Point2D<i32>) -> Point2D<f32> {
        const PIXEL_UNIT: f32 = 16.0;
        Point2D::new((chart_pos.x as f32 * PIXEL_UNIT - chart_pos.y as f32 * PIXEL_UNIT),
                     (chart_pos.x as f32 * PIXEL_UNIT / 2.0 + chart_pos.y as f32 * PIXEL_UNIT / 2.0))
    }
}
