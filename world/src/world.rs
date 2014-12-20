use serialize::json;
use std::cell::RefCell;
use std::rand;
use rand::Rng;
use ecs::Ecs;
use area::Area;
use spatial::Spatial;
use comp::Comp;
use flags::Flags;
use action;

thread_local!(static WORLD_STATE: RefCell<WorldState> = RefCell::new(WorldState::new(None)))

/// Access world state for reading. The world state may not be reaccessed for
/// writing while within this function.
pub fn with<A>(f: |&WorldState| -> A) -> A {
    WORLD_STATE.with(|w| f(w.borrow().deref()))
}

/// Access world state for reading and writing. The world state may not be
/// reaccessed while within this function.
pub fn with_mut<A>(f: |&mut WorldState| -> A) -> A {
    WORLD_STATE.with(|w| f(w.borrow_mut().deref_mut()))
}

/// Save the global world state into a json string.
pub fn save() -> String {
    with(|w| json::encode(w))
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
    pub fn new(seed: Option<u32>) -> WorldState {
        let seed = match seed {
            Some(s) => s,
            // Use system rng for seed if the user didn't provide one.
            None => rand::task_rng().gen()
        };
        WorldState {
            ecs: Ecs::new(),
            area: Area::new(seed, ::AreaSpec::new(::Biome::Overland, 1)),
            spatial: Spatial::new(),
            comp: Comp::new(),
            flags: Flags::new(seed),
        }
    }
}

/// Set up a fresh start-game world state with an optional fixed random number
/// generator seed. Calling init_world will cause any existing world state to
/// be discarded.
pub fn init_world(seed: Option<u32>) {
    WORLD_STATE.with(|w| *w.borrow_mut() = WorldState::new(seed));
    action::start_level(1);
}
