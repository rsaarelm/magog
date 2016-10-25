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

/// Light level value.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Light {
    lum: f32,
}

impl Light {
    pub fn new(lum: f32) -> Light {
        assert!(lum >= 0.0 && lum <= 2.0);
        Light { lum: lum }
    }

    pub fn apply(&self, color: calx_color::Rgba) -> calx_color::Rgba {
        let darkness_color = calx_color::Rgba::new(0.05, 0.10, 0.25, color.a);
        calx_alg::lerp(color * darkness_color, color, self.lum)
    }
}
