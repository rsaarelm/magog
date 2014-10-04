#![crate_name="world"]
#![feature(globs)]
#![feature(macro_rules)]
#![feature(tuple_indexing)]
#![comment = "Display independent world logic for Magog"]

extern crate num;
extern crate rand;
extern crate calx;

pub use geom::{Location, DIR6, DIR8};

mod world;
mod ecs;
mod geom;
