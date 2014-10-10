use std::cell::RefCell;
use std::rc::Rc;
use std::rand;
use rand::Rng;
use ecs::Ecs;
use area::Area;

local_data_key!(WORLD_STATE: Rc<RefCell<WorldState>>)

/// Get the global world instance.
pub fn get() -> Rc<RefCell<WorldState>> {
    if WORLD_STATE.get().is_none() {
        // Lazy init.
        WORLD_STATE.replace(Some(Rc::new(RefCell::new(WorldState::new()))));
        init_world(rand::task_rng().gen());
    }

    WORLD_STATE.get().unwrap().clone()
}

/// The internal object that holds all the world state data.
#[deriving(Encodable, Decodable)]
pub struct WorldState {
    pub ecs: Ecs,
    pub area: Area,
    pub flags: Flags,
}

impl WorldState {
    pub fn new() -> WorldState {
        let seed = 0;
        WorldState {
            ecs: Ecs::new(),
            area: Area::new(seed),
            flags: Flags::new(seed),
        }
    }
}

pub fn init_world(seed: u32) {
    // XXX: Uh-oh. This crashes. How do we do stuff with things accessed from
    // within worldstate doing mutatey stuff on worldstate?
    //get().borrow_mut().area.populate();

    // TODO: Spawn mobs using Area when first starting but not when restoring
    // a save.
}

#[deriving(Encodable, Decodable)]
pub struct Flags {
    pub seed: u32,
    pub populated: bool,
}

impl Flags {
    fn new(seed: u32) -> Flags {
        Flags {
            seed: seed,
            populated: false,
        }
    }
}
