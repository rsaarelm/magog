extern crate rand;
extern crate glium;
extern crate calx_color;
extern crate calx_resource;
extern crate calx_grid;
extern crate world;
extern crate display;

mod init;
pub mod mapgen;

pub use init::{init_brushes, init_terrain};
