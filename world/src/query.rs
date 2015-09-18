/*! Non-mutating world and entity state querying functions. */

use calx_ecs::{Entity};
use world::{World};
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
pub fn control_state(w: &World) -> ControlState {
    if w.flags.player_acted { return ControlState::ReadyToUpdate; }

    match player(w) {
        Some(p) if acts_this_frame(w, p) => ControlState::AwaitingInput,
        _ => ControlState::ReadyToUpdate,
    }
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
pub fn acts_this_frame(w: &World, e: Entity) -> bool {
    if !is_active(w, e) { return false; }
    return ticks_this_frame(w, e);
}

fn brain_state(w: &World, e: Entity) -> Option<BrainState> {
    if !w.ecs.brains.contains(e) { return None; }
    Some(w.ecs.brains[e])
}

/// Return if the entity is a mob that should get an update this frame
/// based on its speed properties. Does not check for status effects like
/// sleep that might prevent actual action.
pub fn ticks_this_frame(w: &World, e: Entity) -> bool {
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
        w.ecs.stats_cache(e).expect("Existing stats_cache value must be Some after cache refresh")
    } else {
        support::base_stats(w, e)
    }
}
