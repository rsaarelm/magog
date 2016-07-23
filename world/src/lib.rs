#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate rand;
extern crate bincode;
extern crate serde;
extern crate vec_map;
extern crate cgmath;
extern crate image;
extern crate euclid;
extern crate vitral;
extern crate calx_alg;
extern crate calx_grid;
extern crate calx_color;
#[macro_use]
extern crate calx_ecs;
#[macro_use]
extern crate calx_resource;

mod brush;
pub use brush::{Brush, BrushBuilder, Color, Frame, ImageRef, Splat};

pub mod item;

mod location;
pub use location::{Location, Chart, Unchart};

mod location_set;
mod spatial;

pub mod terrain;
