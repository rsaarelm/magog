use std::cell::RefCell;
use std::rc::Rc;
use std::rand;
use rand::Rng;
use ecs::Ecs;
use area::Area;
use spawn::{spawn};

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

/// Things you do to a newly-created world but not to one restored from a save
/// file go here.
pub fn init_world(seed: u32) {
    let spawns = get().borrow().area.get_spawns();

    for &(ref seed, ref loc) in spawns.iter() {
        spawn(seed, *loc);
    }
}

#[deriving(Encodable, Decodable)]
pub struct Flags {
    pub seed: u32,
}

impl Flags {
    fn new(seed: u32) -> Flags {
        Flags {
            seed: seed,
        }
    }
}
