use serialize::json;
use std::cell::RefCell;
use std::rand;
use rand::Rng;
use ecs::Ecs;
use area::Area;
use spatial::Spatial;
use comp::Comp;
use flags::Flags;
use flags;
use mob::MobType::{Player};
use egg::Egg;

local_data_key!(WORLD_STATE: RefCell<WorldState>)

fn check_state() {
    if WORLD_STATE.get().is_none() {
        // Lazy init. Use a value from the system rng for the world seed if
        // none was given by the user.
        init_world(None);
    }
}

/// Access world state for reading. The world state may not be reaccessed for
/// writing while within this function.
pub fn with<A>(f: |&WorldState| -> A) -> A {
    check_state();
    f(WORLD_STATE.get().unwrap().borrow().deref())
}

/// Access world state for reading and writing. The world state may not be
/// reaccessed while within this function.
pub fn with_mut<A>(f: |&mut WorldState| -> A) -> A {
    check_state();
    f(WORLD_STATE.get().unwrap().borrow_mut().deref_mut())
}

/// Save the global world state into a json string.
pub fn save() -> String {
    with(|w| json::encode(w))
}

/// Load the global world state from a json string. If the load operation
/// is successful, the previous world state will be overwritten by the loaded
/// one.
pub fn load(json: &str) -> Result<(), json::DecoderError> {
    match json::decode(json) {
        Ok(w) => {
            WORLD_STATE.replace(Some(RefCell::new(w)));
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
    pub fn new(seed: u32) -> WorldState {
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
    let seed = match seed {
        Some(s) => s,
        // Use system rng for seed if the user didn't provide one.
        None => rand::task_rng().gen()
    };

    WORLD_STATE.replace(Some(RefCell::new(WorldState::new(seed))));
    let eggs = with(|w| w.area.get_eggs());
    for &(ref egg, ref loc) in eggs.iter() {
        egg.hatch(*loc);
    }
    let player_entrance = with(|w| w.area.player_entrance());
    Egg::new(::EntityKind::Mob(Player)).hatch(player_entrance);
    flags::set_camera(player_entrance);
}
