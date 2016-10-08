//! Non-mutating world and entity state querying functions.

use std::rc::Rc;
use calx_grid::Dir6;
use calx_ecs::Entity;
use calx_resource::{Resource, ResourceStore};
use world::World;
use components::{Alignment, BrainState};
use stats::{Intrinsic, Stats};
use location::Location;
use spatial::Place;
use item::{ItemType, Slot};
use brush::Brush;
use terrain;
use {FovStatus, Light};

/// Game update control.
#[derive(Copy, Clone, PartialEq)]
pub enum ControlState {
    AwaitingInput,
    ReadyToUpdate,
}

/// Return whether the entity is dead and should be removed from the world.
pub fn is_alive(w: &World, e: Entity) -> bool { location(w, e).is_some() }

/// Return the player entity if one exists.
pub fn player(w: &World) -> Option<Entity> {
    if let Some(p) = w.flags.player {
        if is_alive(w, p) {
            return Some(p);
        }
    }

    None
}

/// Return true if the game has ended and the player can make no further
/// actions.
pub fn game_over(w: &World) -> bool { player(w).is_none() }

/// Get the current control state.
pub fn control_state(w: &World) -> ControlState {
    if w.flags.player_acted {
        return ControlState::ReadyToUpdate;
    }

    let p = player(w);
    if let Some(p) = p {
        if acts_this_frame(w, p) {
            return ControlState::AwaitingInput;
        }
    }

    return ControlState::ReadyToUpdate;
}

/// Return whether this mob is the player avatar.
pub fn is_player(w: &World, e: Entity) -> bool {
    // TODO: Should this just check w.flags.player?
    brain_state(w, e) == Some(BrainState::PlayerControl) && is_alive(w, e)
}

/// Return whether the entity is an awake mob.
pub fn is_active(w: &World, e: Entity) -> bool {
    match brain_state(w, e) {
        Some(BrainState::Asleep) => false,
        Some(_) => true,
        _ => false,
    }
}

/// Return whether the entity is a mob that will act this frame.
pub fn acts_this_frame(w: &World, e: Entity) -> bool {
    if !is_active(w, e) {
        return false;
    }
    return ticks_this_frame(w, e);
}

pub fn brain_state(w: &World, e: Entity) -> Option<BrainState> {
    w.ecs.brain.get(e).map_or(None, |brain| Some(brain.state))
}

/// Return if the entity is a mob that should get an update this frame
/// based on its speed properties. Does not check for status effects like
/// sleep that might prevent actual action.
pub fn ticks_this_frame(w: &World, e: Entity) -> bool {
    if !is_mob(w, e) || !is_alive(w, e) {
        return false;
    }

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
    stats(w, e).intrinsics & intrinsic as u32 != 0
}

/// Return the (composite) stats for an entity.
pub fn stats(w: &World, e: Entity) -> Stats {
    if w.ecs.composite_stats.contains(e) { w.ecs.composite_stats[e].0 } else { base_stats(w, e) }
}

/// Return the base stats of the entity. Does not include any added effects.
/// You almost always want to use the stats function instead of this one.
pub fn base_stats(w: &World, e: Entity) -> Stats {
    if w.ecs.stats.contains(e) { w.ecs.stats[e] } else { Default::default() }
}

pub fn is_mob(w: &World, e: Entity) -> bool { w.ecs.brain.contains(e) }

/// Return the location of an entity.
///
/// Returns the location of the containing entity for entities inside
/// containers. It is possible for entities to not have a location.
pub fn location(w: &World, e: Entity) -> Option<Location> {
    match w.spatial.get(e) {
        Some(Place::At(loc)) => Some(loc),
        Some(Place::In(container, _)) => location(w, container),
        _ => None,
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
            if is_hostile_to(w, shooter, e) {
                return Some(e);
            }
        }
    }
    None
}

/// If location contains a portal, return the destination of the portal.
pub fn portal(w: &World, loc: Location) -> Option<Location> { w.portal(loc).map(|p| loc + p) }

/// Return a portal if it can be seen through.
pub fn visible_portal(w: &World, loc: Location) -> Option<Location> {
    // Only void-form is transparent to portals.
    if terrain(w, loc).form == terrain::Form::Void { portal(w, loc) } else { None }
}

pub fn terrain(w: &World, loc: Location) -> Rc<terrain::Tile> {
    let mut idx = w.terrain.get(loc);

    if idx == 0 {
        use terrain::Id;
        // Empty terrain, inject custom stuff.
        match loc.noise() {
            x if x < 0.5 => idx = Id::Ground as u8,
            x if x < 0.75 => idx = Id::Grass as u8,
            x if x < 0.95 => idx = Id::Water as u8,
            _ => idx = Id::Tree as u8
        }
    }

    terrain::Tile::get_resource(&idx).unwrap()

    // TODO: Add open/closed door mapping to terrain data, closed door terrain should have a field
    // that contains the terrain index of the corresponding open door tile.

    // TODO: Support terrain with brush variant distributions, like the grass case below that
    // occasionlly emits a fancier brush. The distribution needs to be embedded in the Tile struct.
    // The sampling needs loc noise, but is probably better done at the point where terrain is
    // being drawn than here, since we'll want to still have just one immutable terrain id
    // corresponding to all the variants.
    // Mobs standing on doors make the doors open.

    // if ret == TerrainType::Door && has_mobs(w, loc) {
    //     ret = TerrainType::OpenDoor;
    // }
    // // Grass is only occasionally fancy.
    // if ret == TerrainType::Grass {
    //     if loc.noise() > 0.85 {
    //         ret = TerrainType::Grass2;
    //     }
    // }
}

pub fn blocks_sight(w: &World, loc: Location) -> bool { terrain(w, loc).blocks_sight() }

/// Return whether the location obstructs entity movement.
pub fn blocks_walk(w: &World, loc: Location) -> bool {
    if terrain(w, loc).blocks_walk() {
        return true;
    }
    if w.spatial
        .entities_at(loc)
        .into_iter()
        .any(|e| is_blocking_entity(w, e)) {
        return true;
    }
    false
}

/// Return whether the entity blocks movement of other entities.
pub fn is_blocking_entity(w: &World, e: Entity) -> bool { is_mob(w, e) }

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
    if w.ecs.brain.contains(e) { Some(w.ecs.brain[e].alignment) } else { None }
}

/// Return whether the entity can occupy a location.
pub fn can_enter(w: &World, e: Entity, loc: Location) -> bool {
    if is_mob(w, e) && has_mobs(w, loc) {
        return false;
    }
    if terrain(w, loc).is_door() && !has_intrinsic(w, e, Intrinsic::Hands) {
        // Can't open doors without hands.
        return false;
    }
    if blocks_walk(w, loc) {
        return false;
    }
    true
}

/// Return whether the entity can move in a direction.
pub fn can_step(w: &World, e: Entity, dir: Dir6) -> bool {
    if let Some(Place::At(loc)) = w.spatial.get(e) {
        can_enter(w, e, loc + dir.to_v2())
    } else {
        false
    }
}

/// Return the first free storage bag inventory slot on this entity.
pub fn free_bag_slot(w: &World, e: Entity) -> Option<Slot> {
    for &slot in vec![Slot::InventoryJ,
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
                      Slot::InventoryZ]
                     .iter() {
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
    if w.ecs.item.contains(e) { w.ecs.item[e].item_type != ItemType::Instant } else { false }
}

/// Return an item at the location that can be interacted with.
pub fn top_item(w: &World, loc: Location) -> Option<Entity> {
    w.spatial.entities_at(loc).into_iter().find(|&e| can_be_picked_up(w, e))
}

pub fn is_item(w: &World, e: Entity) -> bool { w.ecs.item.contains(e) }

pub fn area_name(w: &World, _loc: Location) -> String {
    match current_depth(w) {
        0 => "Limbo".to_string(),
        1 => "Overworld".to_string(),
        n => format!("Dungeon {}", n - 1),
    }
}

/// Return the current floor depth. Greater depths mean more powerful monsters
/// and stranger terrain.
pub fn current_depth(w: &World) -> i32 { w.flags.depth }

pub fn hp(w: &World, e: Entity) -> i32 {
    max_hp(w, e) - if w.ecs.health.contains(e) { w.ecs.health[e].wounds } else { 0 }
}

pub fn max_hp(w: &World, e: Entity) -> i32 { stats(w, e).power }

pub fn fov_status(w: &World, loc: Location) -> Option<FovStatus> {
    if let Some(p) = player(w) {
        if w.ecs.map_memory.contains(p) {
            if w.ecs.map_memory[p].seen.contains(&loc) {
                return Some(FovStatus::Seen);
            }
            if w.ecs.map_memory[p].remembered.contains(&loc) {
                return Some(FovStatus::Remembered);
            }
            return None;
        }
    }
    // Just show everything by default.
    Some(::FovStatus::Seen)
}

/// Light level for the location.
pub fn light_at(w: &World, loc: Location) -> Light {
    if current_depth(w) == 1 {
        // Topside, full light.
        return Light::new(1.0);
    }
    if terrain(w, loc).is_luminous() {
        return Light::new(1.0);
    }

    if let Some(d) = loc.distance_from(w.flags.camera) {
        let lum = 0.8 - d as f32 / 10.0;
        return Light::new(if lum >= 0.0 { lum } else { 0.0 });
    }
    return Light::new(1.0);
}

/// Return whether the entity is an awake non-player mob and should be
/// animated with a bob.
pub fn is_bobbing(w: &World, e: Entity) -> bool { is_active(w, e) && !is_player(w, e) }

pub fn entity_brush(w: &World, e: Entity) -> Option<Resource<Brush>> {
    if w.ecs.desc.contains(e) { Some(w.ecs.desc[e].brush.clone()) } else { None }
}

pub fn is_instant_item(w: &World, e: Entity) -> bool {
    w.ecs.item.get(e).map_or(false, |item| item.item_type == ItemType::Instant)
}

pub fn can_enter_portals(w: &World, e: Entity) -> bool { is_player(w, e) }
