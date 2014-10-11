#![crate_name="world"]
#![feature(globs)]
#![feature(macro_rules)]
#![feature(tuple_indexing)]
#![comment = "Display independent world logic for Magog"]

extern crate num;
extern crate rand;
extern crate serialize;
extern crate calx;

pub use entity::{Entity};
pub use geom::{HexGeom, DIR6, DIR8};
pub use location::{Location, Chart};
pub use mob::{MobType};
pub use world::{init_world, load, save};

pub mod mapgen;
pub mod terrain;

mod area;
mod comp;
mod entity;
mod ecs;
mod egg;
mod fov;
mod geom;
mod geomorph;
mod geomorph_data;
mod location;
mod mob;
mod spatial;
mod world;

#[deriving(Eq, PartialEq, Show)]
pub enum FovStatus {
    Seen,
    Remembered,
}

/// General type of a game entity.
#[deriving(Eq, PartialEq, Show, Encodable, Decodable)]
pub enum EntityKind {
    /// An active, mobile entity like the player or the NPCs.
    MobKind(MobType),
    /// An entity that can be picked up and used in some way.
    ItemKind, // TODO ItemType data.
    /// A background item that doesn't do much.
    PropKind,
    /// A static object that does things when stepped on.
    NodeKind,
}
