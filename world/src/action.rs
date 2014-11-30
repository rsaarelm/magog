use std::iter::Filter;
use calx::dijkstra::Dijkstra;
use entity::Entity;
use ecs::EntityIter;
use world;
use flags;
use dir6::Dir6;
use area::Area;
use egg::Egg;
use mob::MobType::Player;
use location::Location;

/// Game update control.
#[deriving(PartialEq)]
pub enum ControlState {
    AwaitingInput,
    ReadyToUpdate,
}

/// Player input action.
#[deriving(Eq, PartialEq, Clone, Show, Encodable, Decodable)]
pub enum Input {
    /// Take a step in the given direction.
    Step(Dir6),
    Melee(Dir6),
    // TODO: More
}

/// Return the player entity if one exists.
pub fn player() -> Option<Entity> {
    let mut iter = world::with(|w| w.ecs.iter());
    for e in iter {
        if e.is_player() { return Some(e); }
    }
    None
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
    let p = player().unwrap();
    match input {
        Input::Step(d) => {
            p.step(d);
            flags::set_camera(p.location().unwrap());
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
pub fn mobs<'a>() -> Filter<'a, Entity, EntityIter> {
    entities().filter(|e| e.is_mob())
}

/// Run AI for all autonomous mobs.
fn ai_main() {
    for entity in entities() {
        entity.update();
    }
}

// World logic /////////////////////////////////////////////////////////

/// Return the current floor depth. Greater depths mean more powerful monsters
/// and stranger terrain.
pub fn current_depth() -> int { world::with(|w| w.area.seed.spec.depth) }

pub fn start_level(depth: int) {

    let biome = match depth {
        1 => ::Biome::Overland,
        _ => ::Biome::Dungeon,
    };

    clear_nonplayers();


    let seed = world::with(|w| w.flags.seed);

    world::with_mut(|w| {
        let new_area = Area::new(
            seed,
            ::AreaSpec::new(biome, depth));
        w.area = new_area
    });

    let eggs = world::with(|w| w.area.get_eggs());
    for &(ref egg, ref loc) in eggs.iter() {
        egg.hatch(*loc);
    }

    let start_loc = world::with(|w| w.area.player_entrance());
    // Either reuse the existing player or create a new one.
    match player() {
        Some(p) => {
            p.forget_map();
            p.place(start_loc);
        }
        None => { Egg::new(::EntityKind::Mob(Player)).hatch(start_loc); }
    };
    flags::set_camera(start_loc);
}

fn clear_nonplayers() {
    for e in entities() {
        if !e.is_player() {
            e.delete();
        }
    }
}

/// Move the player to the next level.
pub fn next_level() {
    // This is assuming a really simple, original Rogue style descent-only, no
    // persistent maps style world.
    start_level(current_depth() + 1);
    caption!("Depth {}", current_depth());
}

////////////////////////////////////////////////////////////////////////

pub fn autoexplore_map() -> Option<Dijkstra<Location>> {
    let pathing_depth = 16;

    let locs = world::with(|w| w.area.terrain.iter()
                           .map(|(&loc, _)| loc)
                           .filter(|loc| loc.fov_status().is_none())
                           .collect::<Vec<Location>>());

    if locs.len() == 0 {
        return None;
    }

    Some(Dijkstra::new(locs, |&loc| !loc.blocks_walk(), pathing_depth))
}
