use std::cell::RefCell;
use std::rc::Rc;
use ecs::Ecs;

local_data_key!(WORLD_STATE: Rc<RefCell<WorldState>>)

/// Get the global world instance.
pub fn get() -> Rc<RefCell<WorldState>> {
    if WORLD_STATE.get().is_none() {
        // Lazy init.
        WORLD_STATE.replace(Some(Rc::new(RefCell::new(WorldState::new()))));
    }

    WORLD_STATE.get().unwrap().clone()
}

/// The internal object that holds all the world state data.
pub struct WorldState {
    pub seed: u32,
    pub ecs: Ecs,
}

impl WorldState {
    pub fn new() -> WorldState {
        WorldState {
            seed: 0,
            ecs: Ecs::new(),
        }
    }
}
