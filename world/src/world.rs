use rand;
use rand::Rng;
use rustc_serialize::json;
use content::TerrainType;
use area;
use field::Field;
use spatial::Spatial;
use flags::Flags;
use components;
use stats;

Ecs! {
    desc: components::Desc,
    map_memory: components::MapMemory,
    health: components::Health,
    brain: components::Brain,
    item: components::Item,
    composite_stats: components::CompositeStats,
    stats: stats::Stats,
}

/// Toplevel game state object.
#[derive(RustcEncodable, RustcDecodable)]
pub struct World {
    /// Entity component system.
    pub ecs: Ecs,
    /// Terrain data.
    pub terrain: Field<TerrainType>,
    /// Spatial index for game entities.
    pub spatial: Spatial,
    /// Global gamestate flags.
    pub flags: Flags,
}

impl<'a> World {
    pub fn new(seed: Option<u32>) -> World {
        let seed = match seed {
            // Some RNGs don't like 0 as seed, work around this.
            Some(0) => 1,
            Some(s) => s,
            // Use system rng for seed if the user didn't provide one.
            None => rand::thread_rng().gen(),
        };

        let mut ret = World {
            ecs: Ecs::new(),
            terrain: Field::new(TerrainType::Tree),
            spatial: Spatial::new(),
            flags: Flags::new(seed),
        };

        area::start_level(&mut ret, 1);
        ret
    }

    /// Load a world state from a json string.
    pub fn load(json: &str) -> Result<World, json::DecoderError> {
        json::decode::<World>(json)
    }

    /// Save the global world state into a json string.
    pub fn save(&self) -> String {
        json::encode(self).unwrap()
    }
}
