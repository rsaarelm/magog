use std::cell::RefCell;
use std::rc::Rc;

local_data_key!(WORLD_STATE: Rc<RefCell<WorldState>>)

/// Get the global world instance.
pub fn world() -> World {
    if WORLD_STATE.get().is_none() {
        // Lazy init.
        WORLD_STATE.replace(Some(Rc::new(RefCell::new(WorldState::new()))));
    }

    World(WORLD_STATE.get().unwrap().clone())
}

/// Cloneable handle for world.
#[deriving(Clone)]
pub struct World(pub Rc<RefCell<WorldState>>);

/// The internal object that holds all the world state data.
pub struct WorldState {
    pub seed: u32,
}

impl WorldState {
    pub fn new() -> WorldState {
        WorldState {
            seed: 0,
        }
    }
}
