#![crate_name="world"]
#![feature(globs)]
#![feature(macro_rules)]
#![feature(tuple_indexing)]
#![comment = "Display independent world logic for Magog"]

extern crate num;
extern crate rand;
extern crate calx;

pub use geom::{HexGeom, DIR6, DIR8};
pub use location::{Location, Chart};
pub use world::{init_world};
pub use fov::{FovStatus};

mod ecs;
mod geom;
mod location;
mod world;
pub mod terrain;
pub mod mapgen;
mod geomorph;
mod geomorph_data;
mod fov;
