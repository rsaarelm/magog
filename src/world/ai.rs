use world::world::World;
use world::mobs::*;

pub fn player() -> Option<MobId> { World::map(|w| w.player()) }

pub fn player_has_turn() -> bool {
    match player() {
        Some(p) => p.acts_this_frame(),
        _ => false
    }
}

/// List the mobs that should have an update method called this frame.
pub fn live_mobs() -> Vec<MobId> {
    World::map(|w| w.mobs.iter().map(|(&id, _)| id).collect())
}

pub fn update_mobs() {
    for mob in live_mobs().iter() {
        mob.update_ai();
    }

    World::map_mut(|w| w.advance_frame());
}
