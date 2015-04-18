#![crate_name="mapgen"]

extern crate num;
extern crate rustc_serialize;
extern crate rand;
extern crate calx;

mod geomorph;
mod geomorph_data;
mod mapgen;
mod terrain;

pub use mapgen::{gen_herringbone};
pub use terrain::{TerrainType};

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
        use terrain::TerrainType;
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
