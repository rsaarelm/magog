use crate::{
    ai, animations, components, desc, flags::Flags, item, spatial::Spatial, spec::EntitySpawn,
    stats, world_cache::WorldCache, Distribution, ExternalEntity, Location, Rng, WorldSkeleton,
};
use calx::seeded_rng;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const GAME_VERSION: &str = "0.1.0";

calx_ecs::build_ecs! {
    anim: animations::Anim,
    brain: ai::Brain,
    desc: desc::Desc,
    health: stats::Health,
    item: item::Item,
    map_memory: components::MapMemory,
    stacking: item::Stacking,
    stats: stats::StatsComponent,
    status: stats::Statuses,
}

#[derive(Serialize, Deserialize)]
pub struct WorldSeed {
    pub rng_seed: u32,
    pub world_skeleton: WorldSkeleton,
    pub player_character: ExternalEntity,
}

/// Toplevel game state object.
#[derive(Serialize, Deserialize)]
pub struct World {
    /// Game version. Not mutable in the slightest, but the simplest way to
    /// get versioned save files is to just drop it here.
    pub(crate) version: String,
    /// Entity component system.
    pub(crate) ecs: Ecs,
    /// Static startup game world
    pub(crate) world_cache: WorldCache,
    /// Spawns from worldgen that have been generated in world.
    generated_spawns: HashSet<(Location, EntitySpawn)>,
    /// Spatial index for game entities.
    pub(crate) spatial: Spatial,
    /// Global gamestate flags.
    pub(crate) flags: Flags,
    /// Persistent random number generator.
    pub(crate) rng: Rng,
}

impl World {
    pub fn new(world_seed: &WorldSeed) -> World {
        let mut ret = World {
            version: GAME_VERSION.to_string(),
            ecs: Default::default(),
            world_cache: WorldCache::new(world_seed.rng_seed, world_seed.world_skeleton.clone()),
            generated_spawns: Default::default(),
            spatial: Default::default(),
            flags: Default::default(),
            rng: seeded_rng(&world_seed.rng_seed),
        };

        ret.spawn_player(
            ret.world_cache.player_entrance(),
            &world_seed.player_character,
        );
        ret.generate_world_spawns();

        ret
    }

    pub(crate) fn generate_world_spawns(&mut self) {
        let mut spawns = self.world_cache.drain_spawns();
        spawns.retain(|s| !self.generated_spawns.contains(s));
        let seed = self.rng_seed();

        for (loc, s) in &spawns {
            // Create one-off RNG from just the spawn info, will always run the same for same info.
            let mut rng = calx::seeded_rng(&(seed, loc, s));
            // Construct loadout from the spawn info and generate it in world.
            self.spawn(&s.sample(&mut rng), *loc);
            self.generated_spawns.insert((*loc, s.clone()));
        }
    }
}
