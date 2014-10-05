use std::cell::RefCell;
use std::collections::hashmap::HashMap;
use std::rc::Rc;
use std::rand;
use rand::Rng;
use ecs::Ecs;
use location::Location;
use terrain::TerrainType;
use mapgen;

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
pub struct WorldState {
    pub seed: u32,
    pub ecs: Ecs,
    pub terrain: HashMap<Location, TerrainType>,
}

impl WorldState {
    pub fn new() -> WorldState {
        WorldState {
            seed: 0,
            ecs: Ecs::new(),
            terrain: HashMap::new(),
        }
    }
}

pub fn init_world(seed: u32) {
    // TODO: Proper world init.
    mapgen::gen_herringbone(&Location::new(0, 0), &mapgen::AreaSpec::new(mapgen::Overland, 1));
}
