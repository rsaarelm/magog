#![crate_name="content"]

extern crate num;
extern crate rustc_serialize;
extern crate rand;
extern crate image;
#[macro_use]
extern crate calx;

mod brush;
mod geomorph;
mod geomorph_data;
mod herringbone;
mod rooms;
mod terrain;

use std::collections::BTreeMap;
use calx::V2;

pub use brush::Brush;
pub use herringbone::herringbone;
pub use rooms::rooms_and_corridors;
pub use terrain::TerrainType;

/// Landscape type. Also serves as bit field in order to produce habitat masks
/// for entity spawning etc.
#[derive(Copy, Eq, PartialEq, Clone, Debug, Hash, RustcEncodable, RustcDecodable)]
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

#[derive(Copy, Eq, PartialEq, Debug, Clone, Hash, RustcEncodable, RustcDecodable)]
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

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, RustcEncodable, RustcDecodable)]
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

pub struct StaticArea<T> {
    pub terrain: BTreeMap<V2<i32>, TerrainType>,
    pub spawns: Vec<(V2<i32>, T)>,
    pub player_entrance: V2<i32>,
}

impl<T> StaticArea<T> {
    pub fn new() -> StaticArea<T> {
        StaticArea {
            terrain: BTreeMap::new(),
            spawns: Vec::new(),
            player_entrance: V2(0, 0),
        }
    }

    pub fn map_spawns<U, F>(self, f: F) -> StaticArea<U>
        where F: Fn(T) -> U
    {
        StaticArea {
            terrain: self.terrain,
            spawns: self.spawns.into_iter().map(|(p, x)| (p, f(x))).collect(),
            player_entrance: self.player_entrance,
        }
    }

    pub fn is_open(&self, p: V2<i32>) -> bool {
        if let Some(t) = self.terrain.get(&p) {
            !t.blocks_walk()
        } else {
            false
        }
    }
}
