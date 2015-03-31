use std::cell::RefCell;
use rand;
use rand::Rng;
use rustc_serialize::json;
use super::ecs::{Ecs, Comps};
use super::area::Area;
use super::spatial::Spatial;
use super::flags::Flags;
use super::action;
use super::prototype;
use super::{Biome, AreaSpec};

thread_local!(static WORLD_STATE: RefCell<WorldState> = RefCell::new(WorldState::new(None)));

/// Access world state for reading. The world state may not be reaccessed for
/// writing while within this function.
pub fn with<A, F>(mut f: F) -> A
    where F: FnMut(&WorldState) -> A {
    WORLD_STATE.with(|w| f(& *w.borrow()))
}

/// Access world state for reading and writing. The world state may not be
/// reaccessed while within this function.
pub fn with_mut<A, F>(mut f: F) -> A
    where F: FnMut(&mut WorldState) -> A {
    WORLD_STATE.with(|w| f(&mut *w.borrow_mut()))
}

/// Save the global world state into a json string.
pub fn save() -> String {
    with(|w| json::encode(w).unwrap())
}

/// Load the global world state from a json string. If the load operation
/// is successful, the previous world state will be overwritten by the loaded
/// one.
pub fn load(json: &str) -> Result<(), json::DecoderError> {
    WORLD_STATE.with(|w| {
        match json::decode::<WorldState>(json) {
            Ok(ws) => {
                *w.borrow_mut() = ws;
                Ok(())
            }
            Err(e) => Err(e)
        }
    })
}

/// The internal object that holds all the world state data.
#[derive(RustcEncodable, RustcDecodable)]
pub struct WorldState {
    /// Global entity handler.
    pub ecs: Ecs,
    /// World terrain generation and storage.
    pub area: Area,
    /// Spatial index for game entities.
    pub spatial: Spatial,
    /// Global gamestate flags.
    pub flags: Flags,

    pub comps: Comps,
}

impl<'a> WorldState {
    pub fn new(seed: Option<u32>) -> WorldState {
        let seed = match seed {
            // Some RNGs don't like 0 as seed, work around this.
            Some(0) => 1,
            Some(s) => s,
            // Use system rng for seed if the user didn't provide one.
            None => rand::thread_rng().gen()
        };
        WorldState {
            ecs: Ecs::new(),
            area: Area::new(seed, AreaSpec::new(Biome::Overland, 1)),
            spatial: Spatial::new(),
            flags: Flags::new(seed),
            comps: Comps::new(),
        }
    }
}

/// Set up a fresh start-game world state with an optional fixed random number
/// generator seed. Calling init_world will cause any existing world state to
/// be discarded.
pub fn init_world(seed: Option<u32>) {
    WORLD_STATE.with(|w| *w.borrow_mut() = WorldState::new(seed));
    prototype::init();

    action::start_level(1);
}
