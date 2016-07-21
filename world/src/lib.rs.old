#![crate_name="world"]

#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate rand;
extern crate serde;
extern crate bincode;
extern crate num;
extern crate vec_map;
extern crate cgmath;
#[macro_use]
extern crate calx_ecs;
extern crate calx_alg;
extern crate calx_grid;
extern crate calx_color;

pub use flags::Flags;
pub use location::{Location, Chart, Unchart};
pub use msg::pop_msg;
pub use world::World;
pub use spatial::Spatial;

macro_rules! msg(
    ($($arg:tt)*) => ( ::msg::push(::Msg::Text(format!($($arg)*))))
);

macro_rules! msgln(
    ($($arg:tt)*) => ({
        ::msg::push(::Msg::Text(format!($($arg)*)));
        ::msg::push(::Msg::Text("\n".to_string()));
    })
);

macro_rules! caption(
    ($($arg:tt)*) => ( ::msg::push(::Msg::Caption(format!($($arg)*))))
);

pub mod action;
pub mod area;
pub mod components;
pub mod item;
pub mod query;

mod ability;
mod field;
mod flags;
mod form;
pub mod location;
mod location_set;
mod msg;
mod spatial;
mod stats;
mod world;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FovStatus {
    Seen,
    Remembered,
}

/// Various one-off signals the game sends to the UI layer.
#[derive(Clone, Debug)]
pub enum Msg {
    /// Regular event message
    Text(String),
    /// Important event message to the center of the screen
    Caption(String),
    // TODO: Type of effect.
    Explosion(Location),
    Damage(calx_ecs::Entity),
    Gib(Location),
    Beam(Location, Location),
    /// Beam hitting a wall.
    Sparks(Location),
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
