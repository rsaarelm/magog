/*! Internal utility functions for world and entity state. */

use std::default::Default;
use calx_ecs::{Entity};
use world::{World};
use item::{ItemType, Slot};
use stats::{Stats};
use query;

/// Generate cached stats from base stats and the stats of equipped items.
/// This must be called by any method that accesses the stats_caches
/// component.
pub fn refresh_stats_cache(w: &mut World, e: Entity) {
    // If stats cache doesn't exist, do nothing.
    if !w.ecs.stats_cache.contains(e) { return; }

    // If cache is good, do nothing.
    if w.ecs.stats_cache[e].is_some() { return; }

    let mut stats = base_stats(w, e);
    for &slot in [
        Slot::Body,
        Slot::Feet,
        Slot::Head,
        Slot::Melee,
        Slot::Ranged,
        Slot::TrinketF,
        Slot::TrinketG,
        Slot::TrinketH,
        Slot::TrinketI].iter() {
        if let Some(item) = w.spatial.entity_equipped(e, slot) {
            stats = stats + query::stats(w, item);
        }
    }

    w.ecs.stats_cache[e] = Some(stats);
}

/// Return the base stats, if any, of the entity. Does not try to generate
/// or fetch composite stats.
pub fn base_stats(w: &World, e: Entity) -> Stats {
    if w.ecs.stats.contains(e) {
        w.ecs.stats[e]
    } else {
        Default::default()
    }
}
