/*! Functions for changing world and entity state. */

use std::io::prelude::*;
use std::path::{Path};
use std::fs::{self, File};
use rand::StdRng;
use rand::SeedableRng;
use std::iter::Filter;
use calx::{Dijkstra, Dir6};
use calx_ecs::{Entity};
use world::{World};
use flags;
use area::{Area};
use location::{Location};
use content::{Biome, AreaSpec};
use item::{Slot};
use components::{CompositeStats};
use ::{Msg};
use msg;
use query::{self, ControlState};

/// Player input action.
#[derive(Copy, Eq, PartialEq, Clone, Debug, RustcEncodable, RustcDecodable)]
pub enum Input {
    /// Take a step in the given direction.
    Step(Dir6),
    /// Melee attack in the given direction.
    Melee(Dir6),
    /// Shoot in the given direction.
    Shoot(Dir6),
    /// Do nothing for a turn.
    Pass,
}


/// Top-level game state update function. Only valid to call if
/// control_state() returned ReadyToUpdate.
pub fn update(w: &mut World) {
    assert!(query::control_state(w) == ControlState::ReadyToUpdate);

    ai_main(w);

    w.flags.tick += 1;
    w.flags.player_acted = false;
}

/// Run AI for all autonomous mobs.
fn ai_main(w: &mut World) {
    let actives: Vec<Entity> = w.ecs.brain.iter().map(|(&e, _)| e).collect();
    for e in actives.into_iter() { update_entity(w, e); }
}

pub fn update_entity(w: &mut World, e: Entity) {
    if query::is_mob(w, e) && !query::is_player(w, e) && query::ticks_this_frame(w, e) {
        mob_ai(w, e);
    }
}

pub fn mob_ai(w: &mut World, e: Entity) {
    unimplemented!();
    /*
        assert!(self.is_mob());
        assert!(!self.is_player());
        assert!(self.ticks_this_frame());

        if self.brain_state() == Some(BrainState::Asleep) {
            if let Some(p) = action::player() {
                // TODO: Line-of-sight, stealth concerns, other enemies than
                // player etc.
                if let Some(d) = p.distance_from(self) {
                    if d < 6 {
                        self.wake_up();
                    }
                }
            }

            return;
        }

        if let Some(p) = action::player() {
            let loc = self.location().expect("no location");

            let vec_to_enemy = loc.v2_at(p.location().expect("no location"));
            if let Some(v) = vec_to_enemy {
                if v.hex_dist() == 1 {
                    // Melee range, hit.
                    self.melee(Dir6::from_v2(v));
                } else {
                    // Walk towards.
                    let pathing_depth = 16;
                    let pathing = Dijkstra::new(
                        vec![p.location().expect("no location")], |&loc| !loc.blocks_walk(),
                        pathing_depth);

                    let steps = pathing.sorted_neighbors(&loc);
                    if steps.len() > 0 {
                        self.step(loc.dir6_towards(steps[0]).expect("No loc pair orientation"));
                    } else {
                        self.step(rng().gen());
                        // TODO: Fall asleep if things get boring.
                    }
                }
            }
        }
        */
}

/// Give player input. Only valid to call if control_state() returned
/// AwaitingInput.
pub fn input(w: &mut World, input: Input) {
    assert!(query::control_state(w) == ControlState::AwaitingInput);
    let p = query::player(w).expect("No player to receive input");
    match input {
        Input::Step(d) => {
            step(w, p, d);
        }
        Input::Melee(d) => {
            melee(w, p, d);
        }
        Input::Shoot(d) => {
            shoot(w, p, d);
        }
        Input::Pass => {
        }
    }
    w.flags.player_acted = true;

    // Run one world update cycle right away, so that we don't get awkward
    // single frames rendered where the player has acted and the rest of the
    // world hasn't.
    if query::control_state(w) == ControlState::ReadyToUpdate {
        update(w);
    }
}

/// Try to move the entity in direction.
pub fn step(w: &mut World, e: Entity, dir: Dir6) {
    unimplemented!();
    /*
    let place = world::with(|w| w.spatial.get(self));
    if let Some(Place::At(loc)) = place {
        let new_loc = loc + dir.to_v2();
        if self.can_enter(new_loc) {
            world::with_mut(|w| w.spatial.insert_at(self, new_loc));
            self.on_move_to(new_loc);
        }
    }
    */
}

pub fn melee(w: &mut World, e: Entity, dir: Dir6) {
    unimplemented!();
    /*
    let loc = self.location().expect("no location") + dir.to_v2();
    if let Some(e) = loc.mob_at() {
        let us = self.stats();
        e.damage(us.power + us.attack);
    }
    */
}

pub fn shoot(w: &mut World, e: Entity, dir: Dir6) {
    unimplemented!();
    /*
    let stats = self.stats();

    if stats.ranged_range > 0 {
        action::shoot(self.location().unwrap(), dir, stats.ranged_range, stats.ranged_power);
    }
    */
}

pub fn pick_up(w: &mut World, picker: Entity, item: Entity) -> bool {
    if !query::can_be_picked_up(w, item) {
        return false;
    }

    match query::free_bag_slot(w, picker) {
        Some(slot) => {
            equip(w, item, picker, slot);
            return true;
        }
        // Inventory full.
        None => { return false; }
    }
}

/// Equip an item to a slot. Slot must be empty.
pub fn equip(w: &mut World, item: Entity, e: Entity, slot: Slot) {
    w.spatial.equip(item, e, slot);
    recompose_stats(w, e)
}

/// Generate composed stats from base stats and the stats of equipped items.
/// This function must be called after any operation that changes the composed
/// stats affecting state of an entity.
pub fn recompose_stats(w: &mut World, e: Entity) {
    let mut stats = query::base_stats(w, e);
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

    w.ecs.composite_stats[e] = CompositeStats(stats);
}
/*
/// Return the player entity if one exists.
pub fn player() -> Option<Entity> {
    world::with(|w| w.flags.player)
}

/// Return true if the game has ended and the player can make no further
/// actions.
pub fn is_game_over() -> bool {
    player().is_none()
}

/// Spawn a specific type of entity
pub fn spawn_named(name: &str, loc: Location) -> Result<Entity, ()> {
    unimplemented!();
}

// World update state machine //////////////////////////////////////////

/// Get the current control state.
pub fn control_state() -> ControlState {
    if world::with(|w| w.flags.player_acted) { return ControlState::ReadyToUpdate; }
    match player() {
        Some(p) if p.acts_this_frame() => ControlState::AwaitingInput,
        _ => ControlState::ReadyToUpdate,
    }
}

// Entities ////////////////////////////////////////////////////////////

/// Return an iterator of all the world entities.
pub fn entities() -> EntityIter {
    world::with(|w| w.old_ecs.iter())
}

/// Return an iterator of all the world mobs.
pub fn _mobs() -> Filter<EntityIter, fn(&Entity) -> bool> {
    fn _is_mob(e: &Entity) -> bool { e.is_mob() }
    entities().filter(_is_mob)
}

// World logic /////////////////////////////////////////////////////////

/// Return the current floor depth. Greater depths mean more powerful monsters
/// and stranger terrain.
pub fn current_depth() -> i32 { world::with(|w| w.area.seed.spec.depth) }

pub fn start_level(w: &mut World, depth: i32) {
    let biome = match depth {
        1 => Biome::Overland,
        _ => Biome::Dungeon,
    };

    clear_nonplayers();

    let seed = world::with(|w| w.flags.seed);

    let new_area = Area::new(
        seed,
        AreaSpec::new(biome, depth));
    // XXX: How to move area into the closure without cloning?
    world::with_mut(|w| {
        w.area = new_area.clone();
    });

    /*
    // TODO: Get spawns working again.
    let mut rng: StdRng = SeedableRng::from_seed(&[seed as usize + depth as usize][..]);
    for (spawn, loc) in world::with(|w| w.area.get_spawns()).into_iter() {
        spawn.spawn(&mut rng, loc);
    }
    */

    let start_loc = world::with(|w| w.area.player_entrance());
    // Either reuse the existing player or create a new one.
    match player() {
        Some(p) => {
            p.forget_map();
            p.place(start_loc);
        }
        None => {
            let player = spawn_named("player", start_loc).unwrap();
            world::with_mut(|w| w.flags.player = Some(player));
        }
    };
    flags::set_camera(start_loc);
}

fn clear_nonplayers() {
    let po = player();
    for e in entities() {
        // Don't destroy player or player's inventory.
        if let Some(p) = po {
            if e == p || p.contains(e) {
                continue;
            }
        }

        if e.location().is_some() {
            e.delete();
        }
    }
}

/// Move the player to the next level.
pub fn next_level() {
    // This is assuming a really simple, original Rogue style descent-only, no
    // persistent maps style world.
    start_level(current_depth() + 1);
    caption!("Depth {}", current_depth() - 1);
}

// Effects /////////////////////////////////////////////////////////////

/// Create a projectile arc in dir from origin.
pub fn shoot(origin: Location, dir: Dir6, range: u32, power: i32) {
    let mut loc = origin;
    if range == 0 { return; }
    for i in 1..(range + 1) {
        loc = origin + dir.to_v2() * (i as i32);
        if loc.terrain().blocks_shot() {
            msg::push(::Msg::Sparks(loc));
            break;
        }
        if let Some(e) = loc.mob_at() {
            e.damage(power);
            break;
        }
    }
    msg::push(::Msg::Beam(origin, loc));
}

/// Generate an explosion at location.
pub fn explode(center: Location, power: i32) {
    // Add more complex parametrization if needed.
    msg::push(Msg::Explosion(center));

    // Damage the center.
    if let Some(e) = center.mob_at() {
        e.damage(power);
    }

    for d in Dir6::iter() {
        if let Some(e) = (center + d.to_v2()).mob_at() {
            // Explosions with enough power push back mobs.
            // TODO: Pushback might consider mob size, big guys get pushed
            // around less.
            let push_threshold = 4;
            if power >= push_threshold {
                e.push(*d, ((power - push_threshold) / 2 + 1) as u32);
            }
            if e.exists() {
                e.damage(power);
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////

/// Build a Dijkstra map towards the unexplored corners of the player's FOV.
///
/// Pathing_depth is the depth of the search map. Low pathing depths may not
/// reach distant unexplored cells, but high pathing depths take longer to
/// calculate.
pub fn autoexplore_map(pathing_depth: u32) -> Option<Dijkstra<Location>> {
    let locs = world::with(|w| w.area.terrain.iter()
                           .map(|(&loc, _)| loc)
                           .filter(|loc| loc.fov_status().is_none())
                           .collect::<Vec<Location>>());

    if locs.len() == 0 {
        return None;
    }

    Some(Dijkstra::new(locs, |&loc| !loc.blocks_walk(), pathing_depth))
}

*/
////////////////////////////////////////////////////////////////////////

static SAVE_FILENAME: &'static str = "magog_save.json";

pub fn save_game(w: &World) {
    // Only save if there's still a living player around.
    if query::game_over(w) {
        return;
    }

    let save_data = w.save();
    File::create(SAVE_FILENAME).unwrap()
        .write_all(&save_data.into_bytes()).unwrap();
}

pub fn load_game() -> Result<World, ()> {
    if !save_exists() { return Err(()); }
    let path = Path::new(SAVE_FILENAME);
    let mut save_data = String::new();
    File::open(&path).unwrap().read_to_string(&mut save_data).unwrap();
    // TODO: Informative error message if load fails.
    match World::load(&save_data[..]) {
        Ok(w) => Ok(w),
        _ => Err(())
    }
}

pub fn _delete_save() {
    let _ = fs::remove_file(SAVE_FILENAME);
}

pub fn save_exists() -> bool { fs::metadata(SAVE_FILENAME).is_ok() }

