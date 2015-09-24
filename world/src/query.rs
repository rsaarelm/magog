/*! Non-mutating world and entity state querying functions. */

use calx::{noise, Dir6};
use calx_ecs::{Entity};
use content::TerrainType;
use world::World;
use components::{BrainState, Alignment};
use stats::{Stats, Intrinsic};
use location::Location;
use spatial::{Place};
use item::{ItemType, Slot};
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
pub fn acts_this_frame(w: &World, e: Entity) -> bool {
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

pub fn has_intrinsic(w: &World, e: Entity, intrinsic: Intrinsic) -> bool {
    if !w.ecs.stats_cache.contains(e) { return false; }
    w.ecs.stats_cache[e].map_or(false, |stat| stat.intrinsics & intrinsic as u32 != 0)
}

pub fn stats(w: &World, e: Entity) -> Stats {
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

/// Return whether the location obstructs entity movement.
pub fn blocks_walk(w: &World, loc: Location) -> bool {
    if terrain(w, loc).blocks_walk() { return true; }
    if w.spatial.entities_at(loc).into_iter().any(|e| is_blocking_entity(w, e)) {
        return true;
    }
    false
}

/// Return whether the entity blocks movement of other entities.
pub fn is_blocking_entity(w: &World, e: Entity) -> bool {
    is_mob(w, e)
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

/// Return whether the entity can occupy a location.
pub fn can_enter(w: &World, e: Entity, loc: Location) -> bool {
    if is_mob(w, e) && has_mobs(w, loc) { return false; }
    if terrain(w, loc).is_door() && !has_intrinsic(w, e, Intrinsic::Hands) {
        // Can't open doors without hands.
        return false;
    }
    if blocks_walk(w, loc) { return false; }
    true
}

/// Return whether the entity can move in a direction.
pub fn can_step(w: &World, e: Entity, dir: Dir6) -> bool {
    let place = w.spatial.get(e);
    if let Some(Place::At(loc)) = place {
        let new_loc = loc + dir.to_v2();
        return can_enter(w, e, new_loc);
    }
    return false;
}

/// Return the first free storage bag inventory slot on this entity.
pub fn free_bag_slot(w: &World, e: Entity) -> Option<Slot> {
    for &slot in vec![
        Slot::InventoryJ,
        Slot::InventoryK,
        Slot::InventoryL,
        Slot::InventoryM,
        Slot::InventoryN,
        Slot::InventoryO,
        Slot::InventoryP,
        Slot::InventoryQ,
        Slot::InventoryR,
        Slot::InventoryS,
        Slot::InventoryT,
        Slot::InventoryU,
        Slot::InventoryV,
        Slot::InventoryW,
        Slot::InventoryX,
        Slot::InventoryY,
        Slot::InventoryZ].iter() {
        if equipped(w, e, slot).is_none() {
            return Some(slot);
    }
}
None
}

/// Return the item equipped by this entity in the given inventory slot.
pub fn equipped(w: &World, e: Entity, slot: Slot) -> Option<Entity> {
w.spatial.entity_equipped(e, slot)
}

pub fn can_be_picked_up(w: &World, e: Entity) -> bool {
    if w.ecs.item.contains(e) {
        w.ecs.item[e].item_type != ItemType::Instant
    } else {
        false
    }
}

/// Return an item at the location that can be interacted with.
pub fn top_item(w: &World, loc: Location) -> Option<Entity> {
    w.spatial.entities_at(loc).into_iter().find(|&e| can_be_picked_up(w, e))
}
