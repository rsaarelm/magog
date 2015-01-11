use std::rand::Rng;
use std::rand::distributions::{Weighted, WeightedChoice, IndependentSample};
use std::iter::Filter;
use calx::dijkstra::Dijkstra;
use entity::Entity;
use ecs::EntityIter;
use world;
use flags;
use dir6::Dir6;
use area::Area;
use location::Location;
use Biome;
use components::{Category};

/// Game update control.
#[derive(Copy, PartialEq)]
pub enum ControlState {
    AwaitingInput,
    ReadyToUpdate,
}

/// Player input action.
#[derive(Copy, Eq, PartialEq, Clone, Show, RustcEncodable, RustcDecodable)]
pub enum Input {
    /// Take a step in the given direction.
    Step(Dir6),
    Melee(Dir6),
    // TODO: More
}

/// Return the player entity if one exists.
pub fn player() -> Option<Entity> {
    for e in entities() {
        if e.is_player() { return Some(e); }
    }
    None
}

/// Find the first entity that has a local (not inherited) Desc component with
/// the given name.
pub fn find_prototype(name: &str) -> Option<Entity> {
    world::with(|w|
        entities().find(|&e| {
            if let Some(d) = w.descs().get_local(e) {
                if d.name == name { return true; }
            }
            false
        })
    )
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

/// Top-level game state update function. Only valid to call if
/// control_state() returned ReadyToUpdate.
pub fn update() {
    assert!(control_state() == ControlState::ReadyToUpdate);

    ai_main();

    world::with_mut(|w| {
        w.flags.tick += 1;
        w.flags.player_acted = false;
    });
}

/// Give player input. Only valid to call if control_state() returned
/// AwaitingInput.
pub fn input(input: Input) {
    assert!(control_state() == ControlState::AwaitingInput);
    let p = player().expect("No player to receive input");
    match input {
        Input::Step(d) => {
            p.step(d);
            flags::set_camera(p.location().expect("No player location"));
        }
        Input::Melee(d) => {
            p.melee(d);
        }
    }
    world::with_mut(|w| w.flags.player_acted = true);

    // Run one world update cycle right away, so that we don't get awkward
    // single frames rendered where the player has acted and the rest of the
    // world hasn't.
    if control_state() == ControlState::ReadyToUpdate {
        update();
    }
}

// Entities ////////////////////////////////////////////////////////////

/// Return an iterator of all the world entities.
pub fn entities() -> EntityIter {
    world::with(|w| w.ecs.iter())
}

/// Return an iterator of all the world mobs.
pub fn mobs() -> Filter<Entity, EntityIter, fn(&Entity) -> bool> {
    fn is_mob(e: &Entity) -> bool { e.is_mob() }
    entities().filter(is_mob as fn(&Entity) -> bool)
}

/// Run AI for all autonomous mobs.
fn ai_main() {
    for entity in entities() {
        entity.update();
    }
}

/// Spawn n random entities with the give limiting parameters.
pub fn random_spawns<R: Rng>(
    rng: &mut R, count: uint,
    depth: uint, biome_mask: Biome, category_mask: Category)
    -> Vec<Entity> {
    let mut items: Vec<Weighted<Entity>> = entities()
        .filter_map(|e| world::with(|w| {
            if let Some(spawn) = w.spawns().get_local(e) {
                if spawn.min_depth <= depth
                    && (biome_mask as u32) & (spawn.biome as u32) != 0
                    && (category_mask as u32) & (spawn.category as u32) != 0 {
                    return Some(Weighted { weight: spawn.commonness, item: e });
                }
            }
            return None;
        }))
        .collect();
    let dist = WeightedChoice::new(items.as_mut_slice());
    range(0, count).map(|_| dist.ind_sample(rng)).collect()
}

// World logic /////////////////////////////////////////////////////////

/// Return the current floor depth. Greater depths mean more powerful monsters
/// and stranger terrain.
pub fn current_depth() -> int { world::with(|w| w.area.as_ref().expect("no area").seed.spec.depth) }

pub fn start_level(depth: int) {
    let biome = match depth {
        1 => ::Biome::Overland,
        _ => ::Biome::Dungeon,
    };

    clear_nonplayers();

    let seed = world::with(|w| w.flags.seed);

    let new_area = Area::new(
        seed,
        ::AreaSpec::new(biome, depth));
    // XXX: How to move area into the closure without cloning?
    world::with_mut(|w| {
        w.area = Some(new_area.clone())
    });

    for (spawn, loc) in  world::with(|w| w.area.as_ref().expect("no area").get_spawns()).into_iter() {
        spawn.clone_at(loc);
    }

    let start_loc = world::with(|w| w.area.as_ref().expect("no area").player_entrance());
    // Either reuse the existing player or create a new one.
    match player() {
        Some(p) => {
            p.forget_map();
            p.place(start_loc);
        }
        None => {
            find_prototype("player").expect("No Player prototype found!")
            .clone_at(start_loc);
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

////////////////////////////////////////////////////////////////////////

/// Build a Dijkstra map towards the unexplored corners of the player's FOV.
///
/// Pathing_depth is the depth of the search map. Low pathing depths may not
/// reach distant unexplored cells, but high pathing depths take longer to
/// calculate.
pub fn autoexplore_map(pathing_depth: uint) -> Option<Dijkstra<Location>> {
    let locs = world::with(|w| w.area.as_ref().expect("no area").terrain.iter()
                           .map(|(&loc, _)| loc)
                           .filter(|loc| loc.fov_status().is_none())
                           .collect::<Vec<Location>>());

    if locs.len() == 0 {
        return None;
    }

    Some(Dijkstra::new(locs, |&loc| !loc.blocks_walk(), pathing_depth))
}
