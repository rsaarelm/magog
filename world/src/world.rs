use serialize::json;
use std::cell::RefCell;
use std::rc::Rc;
use std::rand;
use rand::Rng;
use ecs::Ecs;
use area::Area;
use spatial::Spatial;
use comp::Comp;

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

/// Save the global world state into a json string.
pub fn save() -> String {
    json::encode(get().borrow().deref())
}

/// Load the global world state from a json string. If the load operation
/// is successful, the previous world state will be overwritten by the loaded
/// one.
pub fn load(json: &str) -> Result<(), json::DecoderError> {
    match json::decode(json) {
        Ok(w) => {
            WORLD_STATE.replace(Some(Rc::new(RefCell::new(w))));
            Ok(())
        }
        Err(e) => Err(e)
    }
}

/// The internal object that holds all the world state data.
#[deriving(Encodable, Decodable)]
pub struct WorldState {
    /// Global entity handler.
    pub ecs: Ecs,
    /// World terrain generation and storage.
    pub area: Area,
    /// Spatial index for game entities.
    pub spatial: Spatial,
    /// Generic components for game entities.
    pub comp: Comp,
    /// Global gamestate flags.
    pub flags: Flags,
}

impl WorldState {
    pub fn new() -> WorldState {
        let seed = 0;
        WorldState {
            ecs: Ecs::new(),
            area: Area::new(seed),
            spatial: Spatial::new(),
            comp: Comp::new(),
            flags: Flags::new(seed),
        }
    }
}

/// Things you do to a newly-created world but not to one restored from a save
/// file go here.
pub fn init_world(seed: u32) {
    let eggs = get().borrow().area.get_eggs();

    for &(ref egg, ref loc) in eggs.iter() {
        egg.hatch(*loc);
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
