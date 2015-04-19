#![crate_name="world"]
#![feature(unboxed_closures, plugin)]
#![feature(core, collections, path_ext)]
#![feature(custom_derive)]
#![plugin(rand_macros)]

#[no_link] extern crate rand_macros;
extern crate rand;
extern crate rustc_serialize;
extern crate num;
extern crate calx;
extern crate mapgen;

pub use entity::{Entity};
pub use flags::{camera, set_camera, get_tick};
pub use location::{Location, Chart, Unchart};
pub use msg::{pop_msg};
pub use world::{init_world, load, save};

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
pub mod components;
pub mod item;

mod ability;
mod area;
mod component_ref;
mod ecs;
mod entity;
mod flags;
pub mod location;
mod location_set;
mod msg;
mod prototype;
mod spatial;
mod spawn;
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
    Damage(Entity),
    Gib(Location),
    Beam(Location, Location),
    /// Beam hitting a wall.
    Sparks(Location),
}

/// Light level value.
#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Light {
    lum: f32,
}

impl Light {
    pub fn new(lum: f32) -> Light {
        assert!(lum >= 0.0 && lum <= 2.0);
        Light { lum: lum }
    }

    pub fn apply(&self, color: &calx::Rgb) -> calx::Rgb {
        if self.lum <= 1.0 {
            // Make the darkness blue instead of totally black.
            calx::Rgb::new(
                (color.r as f32 * calx::clamp(0.0, 1.0, self.lum + 0.125)) as u8,
                (color.g as f32 * calx::clamp(0.0, 1.0, self.lum + 0.25)) as u8,
                (color.b as f32 * calx::clamp(0.0, 1.0, self.lum + 0.5)) as u8)
        } else {
            calx::Rgb::new(
                255 - ((255 - color.r) as f32 * (2.0 - self.lum)) as u8,
                255 - ((255 - color.g) as f32 * (2.0 - self.lum)) as u8,
                255 - ((255 - color.b) as f32 * (2.0 - self.lum)) as u8)
        }
    }
}
