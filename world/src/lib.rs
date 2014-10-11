#![crate_name="world"]
#![feature(globs)]
#![feature(macro_rules)]
#![feature(tuple_indexing)]
#![comment = "Display independent world logic for Magog"]

extern crate num;
extern crate rand;
extern crate serialize;
extern crate calx;

pub use geom::{HexGeom, DIR6, DIR8};
pub use location::{Location, Chart};
pub use world::{init_world};
pub use ecs::{Entity};

pub mod mapgen;
pub mod terrain;

mod area;
mod comp;
mod ecs;
mod fov;
mod geom;
mod geomorph;
mod geomorph_data;
mod location;
mod spatial;
mod spawn;
mod world;

#[deriving(Eq, PartialEq, Show)]
pub enum FovStatus {
    Seen,
    Remembered,
}
