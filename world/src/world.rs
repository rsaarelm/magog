use std::cell::RefCell;
use std::rand;
use std::default::Default;
use rustc_serialize::json;
use rand::Rng;
use util::color;
use ecs::{Ecs, Comps};
use area::Area;
use spatial::Spatial;
use flags::Flags;
use action;
use prototype::{Prototype};
use components::{Spawn, Category};
use components::{Desc, MapMemory, Health};
use components::{Brain, BrainState, Alignment};
use components::{Item};
use item::{ItemType};
use stats::{Stats, Intrinsic};
use ability::Ability;
use {Biome};

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
            Some(s) => s,
            // Use system rng for seed if the user didn't provide one.
            None => rand::thread_rng().gen()
        };
        WorldState {
            ecs: Ecs::new(),
            area: Area::new(seed, ::AreaSpec::new(::Biome::Overland, 1)),
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

    /*
//  Symbol   power, depth, biome, sprite, color,        intrinsics
    Player:     6,  -1, Anywhere, 51, &AZURE,            f!(Hands);
    Dreg:       1,   1, Anywhere, 72, &OLIVE,            f!(Hands);
    Snake:      1,   1, Overland, 71, &GREEN,            f!();
    Ooze:       3,   3, Dungeon,  77, &LIGHTSEAGREEN,    f!();
    Flayer:     4,   4, Anywhere, 75, &INDIANRED,        f!();
    Ogre:       6,   5, Anywhere, 73, &DARKSLATEGRAY,    f!(Hands);
    Wraith:     8,   6, Dungeon,  74, &HOTPINK,          f!(Hands);
    Octopus:    10,  7, Anywhere, 63, &DARKTURQUOISE,    f!();
    Efreet:     12,  8, Anywhere, 78, &ORANGE,           f!();
    Serpent:    15,  9, Dungeon,  94, &CORAL,            f!();
    */

    let base_mob = Prototype::new(None)
        (Brain { state: BrainState::Asleep, alignment: Alignment::Evil })
        ({let h: Health = Default::default(); h})
        .target;

    // Init the prototypes

    // Player
    Prototype::new(Some(base_mob))
        (Brain { state: BrainState::PlayerControl, alignment: Alignment::Good })
        (Desc::new("player", 51, color::AZURE))
        (Stats { power: 6, intrinsics: Intrinsic::Hands as u32 })
        (MapMemory::new())
        ;

    // Enemies
    Prototype::new(Some(base_mob))
        (Desc::new("dreg", 72, color::OLIVE))
        (Stats { power: 1, intrinsics: Intrinsic::Hands as u32 })
        (Spawn { biome: Biome::Anywhere, commonness: 1000, min_depth: 1, category: Category::Mob })
        ;

    Prototype::new(Some(base_mob))
        (Desc::new("snake", 71, color::GREEN))
        (Stats { power: 1, intrinsics: 0 })
        (Spawn { biome: Biome::Overland, commonness: 1000, min_depth: 1, category: Category::Mob })
        ;

    Prototype::new(Some(base_mob))
        (Desc::new("ooze", 77, color::LIGHTSEAGREEN))
        (Stats { power: 3, intrinsics: 0 })
        (Spawn { biome: Biome::Dungeon, commonness: 1000, min_depth: 3, category: Category::Mob })
        ;
    // TODO: More mob types

    // Items
    Prototype::new(None)
        (Desc::new("heart", 89, color::RED))
        (Spawn { biome: Biome::Anywhere, commonness: 1000, min_depth: 1, category: Category::Consumable })
        (Item { item_type: ItemType::Instant, ability: Ability::HealInstant(2) })
        ;

    Prototype::new(None)
        (Desc::new("sword", 84, color::GAINSBORO))
        (Spawn { biome: Biome::Anywhere, commonness: 500, min_depth: 1, category: Category::Equipment })
        (Item { item_type: ItemType::MeleeWeapon, ability: Ability::Multi(vec![]) })
        (Stats { power: 5, intrinsics: 0 })
        ;

    action::start_level(1);
}
