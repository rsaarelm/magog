#![crate_name="content"]

#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate num;
extern crate rand;
extern crate image;
extern crate serde;

#[macro_use]
extern crate calx_alg;

#[macro_use]
extern crate calx_cache;

extern crate calx_grid;
extern crate calx_layout;

mod brush;
mod geomorph;
mod geomorph_data;
mod herringbone;
// mod rooms;
mod terrain;

use std::collections::BTreeMap;

pub use brush::Brush;
pub use herringbone::herringbone;
// pub use rooms::rooms_and_corridors;
pub use terrain::TerrainType;

/// Landscape type. Also serves as bit field in order to produce habitat masks
/// for entity spawning etc.
#[derive(Copy, Eq, PartialEq, Clone, Debug, Hash, Serialize, Deserialize)]
pub enum Biome {
    Overland = 0b1,
    Dungeon = 0b10,

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

    pub fn intersects(self, other: Biome) -> bool {
        (self as u32) & (other as u32) != 0
    }
}

#[derive(Copy, Eq, PartialEq, Debug, Clone, Hash, Serialize, Deserialize)]
pub struct AreaSpec {
    pub biome: Biome,
    pub depth: i32,
}

impl AreaSpec {
    pub fn new(biome: Biome, depth: i32) -> AreaSpec {
        AreaSpec {
            biome: biome,
            depth: depth,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Serialize, Deserialize)]
pub enum FormType {
    Anything,

    Creature,

    Item,

    Consumable,
    Equipment,
}

impl FormType {
    /// Return whether a FormType is a subtype of another type.
    pub fn is_a(self, other: FormType) -> bool {
        use FormType::*;

        match (self, other) {
            (x, y) if x == y => true,
            (_, Anything) => true,
            (Consumable, Item) => true,
            (Equipment, Item) => true,
            _ => false,
        }
    }
}

pub struct StaticArea {
    pub terrain: BTreeMap<[i32; 2], TerrainType>,
    pub spawns: Vec<([i32; 2], FormType)>,
    pub player_entrance: [i32; 2],
}

impl StaticArea {
    pub fn new() -> StaticArea {
        StaticArea {
            terrain: BTreeMap::new(),
            spawns: Vec::new(),
            player_entrance: [0, 0],
        }
    }

    pub fn is_open(&self, p: [i32; 2]) -> bool {
        if let Some(t) = self.terrain.get(&p) {
            !t.blocks_walk()
        } else {
            false
        }
    }
}
