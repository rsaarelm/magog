use std::io::{Read, Write};
use rand;
use rand::Rng;
use bincode::{self, serde};
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
#[derive(Serialize, Deserialize)]
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

    pub fn load<R: Read>(reader: &mut R) -> serde::DeserializeResult<World> {
        serde::deserialize_from(reader, bincode::SizeLimit::Infinite)
    }

    pub fn save<W: Write>(&self, writer: &mut W) -> serde::SerializeResult<()> {
        serde::serialize_into(writer, self, bincode::SizeLimit::Infinite)
    }
}

#[cfg(test)]
mod test {
    use super::World;

    #[test]
    fn test_serialize() {
        use bincode::{serde, SizeLimit};

        let w1 = World::new(Some(123));
        let saved = serde::serialize(&w1, SizeLimit::Infinite).expect("Serialization failed");
        let w2: World = serde::deserialize(&saved).expect("Deserialization failed")
        assert!(w1.flags.seed == w2.flags.seed);
    }
}
