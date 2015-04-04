use calx;

pub use ::world::entity::{Entity};
pub use ::world::flags::{camera, set_camera, get_tick};
pub use ::world::fov::{Fov};
pub use ::world::geom::{HexGeom};
pub use ::world::location::{Location, Chart, Unchart};
pub use ::world::msg::{pop_msg};
pub use ::world::terrain::{TerrainType};
pub use ::world::world::{init_world, load, save};
pub use ::world::dir6::Dir6;

macro_rules! msg(
    ($($arg:tt)*) => ( ::world::msg::push(::world::Msg::Text(format!($($arg)*))))
);

macro_rules! msgln(
    ($($arg:tt)*) => ({
        ::world::msg::push(::world::Msg::Text(format!($($arg)*)));
        ::world::msg::push(::world::Msg::Text("\n".to_string()));
    })
);

macro_rules! caption(
    ($($arg:tt)*) => ( ::world::msg::push(::world::Msg::Caption(format!($($arg)*))))
);

pub mod action;
pub mod components;
pub mod item;

mod ability;
mod area;
mod component_ref;
mod dir6;
mod ecs;
mod entity;
mod flags;
mod fov;
mod geom;
mod geomorph;
mod geomorph_data;
pub mod location;
mod location_set;
mod mapgen;
mod msg;
mod prototype;
mod rng;
mod spatial;
mod spawn;
mod stats;
mod terrain;
mod world;

#[derive(Copy, Eq, PartialEq, Debug)]
pub enum FovStatus {
    Seen,
    Remembered,
}

/// Landscape type. Also serves as bit field in order to produce habitat masks
/// for entity spawning etc.
#[derive(Copy, Eq, PartialEq, Clone, Debug, RustcEncodable, RustcDecodable)]
pub enum Biome {
    Overland = 0b1,
    Dungeon  = 0b10,

    // For things showing up at a biome.
    Anywhere = -1,
}

impl Biome {
    pub fn default_terrain(self) -> terrain::TerrainType {
        match self {
            Biome::Overland => TerrainType::Tree,
            Biome::Dungeon => TerrainType::Rock,
            _ => TerrainType::Void,
        }
    }
}

#[derive(Copy, Eq, PartialEq, Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct AreaSpec {
    pub biome: Biome,
    pub depth: i32,
}

impl AreaSpec {
    pub fn new(biome: Biome, depth: i32) -> AreaSpec {
        AreaSpec { biome: biome, depth: depth }
    }
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
#[derive(Copy, Debug, RustcEncodable, RustcDecodable)]
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
