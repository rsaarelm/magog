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

mod ability;

mod brush;
pub use brush::{Brush, BrushBuilder, Color, Frame, ImageRef, Splat};

pub mod components;
mod field;
mod flags;
mod fov;
pub mod item;

mod location;
pub use location::{Location, Portal};

mod location_set;

mod query;
pub use query::Query;

mod spatial;
mod stats;
pub mod terrain;

mod world;
pub use world::World;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FovStatus {
    Seen,
    Remembered,
}
