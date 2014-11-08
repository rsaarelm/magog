use std::iter::Filter;
use entity::Entity;
use ecs::EntityIter;
use world;
use flags;
use dir6::Dir6;

/// Game update control.
#[deriving(PartialEq)]
pub enum ControlState {
    AwaitingInput,
    ReadyToUpdate,
}

/// Player input action.
#[deriving(Eq, PartialEq, Clone, Show, Encodable, Decodable)]
pub enum PlayerInput {
    /// Take a step in the given direction.
    Step(Dir6),
    // TODO: More
}

/// Return the player entity if one exists.
pub fn player() -> Option<Entity> {
    let mut iter = world::get().borrow().ecs.iter();
    for e in iter {
        if e.is_player() { return Some(e); }
    }
    None
}

/// Get the current control state.
pub fn control_state() -> ControlState {
    if world::get().borrow().flags.player_acted { return ReadyToUpdate; }
    match player() {
        Some(p) if p.acts_this_frame() => AwaitingInput,
        _ => ReadyToUpdate,
    }
}

/// Top-level game state update function. Only valid to call if
/// control_state() returned ReadyToUpdate.
pub fn update() {
    assert!(control_state() == ReadyToUpdate);

    ai_main();

    world::get().borrow_mut().flags.tick += 1;
    world::get().borrow_mut().flags.player_acted = false;
}

/// Give player input. Only valid to call if control_state() returned
/// AwaitingInput.
pub fn input(input: PlayerInput) {
    assert!(control_state() == AwaitingInput);
    let p = player().unwrap();
    match input {
        Step(d) => {
            p.step(d);
            flags::set_camera(p.location().unwrap());
        }
    }
    world::get().borrow_mut().flags.player_acted = true;

    // Run one world update cycle right away, so that we don't get awkward
    // single frames rendered where the player has acted and the rest of the
    // world hasn't.
    if control_state() == ReadyToUpdate {
        update();
    }
}

pub fn entities() -> EntityIter {
    world::get().borrow().ecs.iter()
}

pub fn mobs<'a>() -> Filter<'a, Entity, EntityIter> {
    world::get().borrow().ecs.iter().filter(|e| e.is_mob())
}

/// Run AI for all autonomous mobs.
fn ai_main() {
    for entity in entities() {
        entity.update();
    }
}
