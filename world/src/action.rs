use rand::StdRng;
use rand::SeedableRng;
use std::iter::Filter;
use util::Dijkstra;
use entity::Entity;
use ecs::EntityIter;
use world;
use flags;
use dir6::Dir6;
use area::Area;
use location::Location;
use ecs::{ComponentAccess};
use msg;

/// Game update control.
#[derive(Copy, PartialEq)]
pub enum ControlState {
    AwaitingInput,
    ReadyToUpdate,
}

/// Player input action.
#[derive(Copy, Eq, PartialEq, Clone, Debug, RustcEncodable, RustcDecodable)]
pub enum Input {
    /// Take a step in the given direction.
    Step(Dir6),
    Melee(Dir6),
    Shoot(Dir6),
    // TODO: More
}

/// Return the player entity if one exists.
pub fn player() -> Option<Entity> {
    world::with(|w| w.flags.player)
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

/// Spawn a specific type of entity
pub fn spawn_named(name: &str, loc: Location) {
    find_prototype(name).expect(&format!("Spawn prototype '{}' not found", name)[..])
        .clone_at(loc);
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
        }
        Input::Melee(d) => {
            p.melee(d);
        }
        Input::Shoot(d) => {
            p.shoot(d);
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
pub fn mobs() -> Filter<EntityIter, fn(&Entity) -> bool> {
    fn is_mob(e: &Entity) -> bool { e.is_mob() }
    entities().filter(is_mob as fn(&Entity) -> bool)
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
pub fn current_depth() -> i32 { world::with(|w| w.area.seed.spec.depth) }

pub fn start_level(depth: i32) {
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
        w.area = new_area.clone();
    });

    let mut rng: StdRng = SeedableRng::from_seed(&[seed as usize + depth as usize][..]);
    for (spawn, loc) in world::with(|w| w.area.get_spawns()).into_iter() {
        spawn.spawn(&mut rng, loc);
    }

    let start_loc = world::with(|w| w.area.player_entrance());
    // Either reuse the existing player or create a new one.
    match player() {
        Some(p) => {
            p.forget_map();
            p.place(start_loc);
        }
        None => {
            let player = find_prototype("player").expect("No Player prototype found!")
            .clone_at(start_loc);
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

/// Look for targets to shoot in a direction.
pub fn find_target(shooter: Entity, dir: Dir6, range: usize) -> Option<Entity> {
    let origin = shooter.location().unwrap();
    for i in 1..(range + 1) {
        let loc = origin + dir.to_v2() * (i as i32);
        if loc.terrain().blocks_shot() {
            break;
        }
        if let Some(e) = loc.mob_at() {
            if shooter.is_hostile_to(e) { return Some(e); }
        }
    }
    None
}
