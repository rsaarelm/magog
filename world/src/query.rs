/*! Non-mutating world and entity state querying functions. */

use calx::{noise, Dir6};
use calx_ecs::{Entity};
use content::TerrainType;
use world::World;
use components::{BrainState, Alignment};
use stats::{Stats, Intrinsic};
use location::Location;
use spatial::{Place};
use support;

/// Game update control.
#[derive(Copy, Clone, PartialEq)]
pub enum ControlState {
    AwaitingInput,
    ReadyToUpdate,
}

/// Return the player entity if one exists.
pub fn player(w: &World) -> Option<Entity> { w.flags.player }

/// Return true if the game has ended and the player can make no further
/// actions.
pub fn game_over(w: &World) -> bool { player(w).is_none() }

/// Get the current control state.
pub fn control_state(w: &mut World) -> ControlState {
    if w.flags.player_acted { return ControlState::ReadyToUpdate; }

    let p = player(w);
    if let Some(p) = p {
        if acts_this_frame(w, p) { return ControlState::AwaitingInput; }
    }

    return ControlState::ReadyToUpdate;
}

/// Return whether the entity is an awake mob.
pub fn is_active(w: &World, e: Entity) -> bool {
    match brain_state(w, e) {
        Some(BrainState::Asleep) => false,
        Some(_) => true,
        _ => false
    }
}

/// Return whether the entity is a mob that will act this frame.
pub fn acts_this_frame(w: &mut World, e: Entity) -> bool {
    if !is_active(w, e) { return false; }
    return ticks_this_frame(w, e);
}

fn brain_state(w: &World, e: Entity) -> Option<BrainState> {
    if !w.ecs.brain.contains(e) { return None; }
    Some(w.ecs.brain[e].state)
}

/// Return if the entity is a mob that should get an update this frame
/// based on its speed properties. Does not check for status effects like
/// sleep that might prevent actual action.
pub fn ticks_this_frame(w: &mut World, e: Entity) -> bool {
    if !is_mob(w, e) { return false; }

    let tick = w.flags.tick;
    // Go through a cycle of 5 phases to get 4 possible speeds.
    // System idea from Jeff Lait.
    let phase = tick % 5;
    match phase {
        0 => return true,
        1 => return has_intrinsic(w, e, Intrinsic::Fast),
        2 => return true,
        3 => return has_intrinsic(w, e, Intrinsic::Quick),
        4 => return !has_intrinsic(w, e, Intrinsic::Slow),
        _ => panic!("Invalid action phase"),
    }
}

pub fn has_intrinsic(w: &mut World, e: Entity, intrinsic: Intrinsic) -> bool {
    if !w.ecs.stats_cache.contains(e) { return false; }

    support::refresh_stats_cache(w, e);
    w.ecs.stats_cache[e].map_or(false, |stat| stat.intrinsics & intrinsic as u32 != 0)
}

pub fn stats(w: &mut World, e: Entity) -> Stats {
    support::refresh_stats_cache(w, e);
    if w.ecs.stats_cache.contains(e) {
        w.ecs.stats_cache[e].expect("Existing stats_cache value must be Some after cache refresh")
    } else {
        support::base_stats(w, e)
    }
}

pub fn is_mob(w: &World, e: Entity) -> bool {
    w.ecs.brain.contains(e) && location(w, e).is_some()
}

/// Return the location of an entity.
///
/// Returns the location of the containing entity for entities inside
/// containers. It is possible for entities to not have a location.
pub fn location(w: &World, e: Entity) -> Option<Location> {
    match w.spatial.get(e) {
        Some(Place::At(loc)) => Some(loc),
        Some(Place::In(container, _)) => location(w, container),
        _ => None
    }
}

/// Look for targets to shoot in a direction.
pub fn find_target(w: &World, shooter: Entity, dir: Dir6, range: usize) -> Option<Entity> {
    let origin = location(w, shooter).unwrap();
    for i in 1..(range + 1) {
        let loc = origin + dir.to_v2() * (i as i32);
        if terrain(w, loc).blocks_shot() {
            break;
        }
        if let Some(e) = mob_at(w, loc) {
            if is_hostile_to(w, shooter, e) { return Some(e); }
        }
    }
    None
}

pub fn terrain(w: &World, loc: Location) -> TerrainType {
    let mut ret = w.area.terrain(loc);
    // Mobs standing on doors make the doors open.
    if ret == TerrainType::Door && has_mobs(w, loc) {
        ret = TerrainType::OpenDoor;
    }
    // Grass is only occasionally fancy.
    // TODO: Make variant tiles into a generic method.
    if ret == TerrainType::Grass {
        if loc.noise() > 0.85 {
            ret = TerrainType::Grass2;
        }
    }
    ret
}

pub fn has_mobs(w: &World, loc: Location) -> bool { mob_at(w, loc).is_some() }

pub fn mob_at(w: &World, loc: Location) -> Option<Entity> {
    w.spatial.entities_at(loc).into_iter().find(|&e| is_mob(w, e))
}

pub fn is_hostile_to(w: &World, e: Entity, other: Entity) -> bool {
    match (alignment(w, e), alignment(w, other)) {
        (Some(Alignment::Chaotic), Some(_)) => true,
        (Some(_), Some(Alignment::Chaotic)) => true,
        (Some(Alignment::Evil), Some(Alignment::Good)) => true,
        (Some(Alignment::Good), Some(Alignment::Evil)) => true,
        _ => false,
    }
}

pub fn alignment(w: &World, e: Entity) -> Option<Alignment> {
    if w.ecs.brain.contains(e) {
        Some(w.ecs.brain[e].alignment)
    } else {
        None
    }
}
