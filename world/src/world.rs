use std::collections::VecMap;
use std::cell::RefCell;
use std::rand;
use std::default::Default;
use rustc_serialize::json;
use rand::Rng;
use calx::color;
use ecs::Ecs;
use area::Area;
use spatial::Spatial;
use flags::Flags;
use entity::Entity;
use action;
use components::{Prototype};
use components::{StatsCache};
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
#[deriving(RustcEncodable, RustcDecodable)]
pub struct WorldState {
    /// Global entity handler.
    pub ecs: Ecs,
    /// World terrain generation and storage.
    pub area: Option<Area>,
    /// Spatial index for game entities.
    pub spatial: Spatial,
    /// Global gamestate flags.
    pub flags: Flags,

    comps: Comps,
}

impl<'a> WorldState {
    pub fn new(seed: Option<u32>) -> WorldState {
        let seed = match seed {
            Some(s) => s,
            // Use system rng for seed if the user didn't provide one.
            None => rand::task_rng().gen()
        };
        WorldState {
            ecs: Ecs::new(),
            area: None,
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
        (Desc { name: "player".to_string(), icon: 51, color: color::AZURE })
        (Stats { power: 6, intrinsics: Intrinsic::Hands as u32 })
        (MapMemory::new())
        ;

    // Enemies
    Prototype::new(Some(base_mob))
        (Desc { name: "dreg".to_string(), icon: 72, color: color::OLIVE })
        (Stats { power: 1, intrinsics: Intrinsic::Hands as u32 })
        (Spawn { biome: Biome::Anywhere, commonness: 1000, min_depth: 1, category: Category::Mob })
        ;

    Prototype::new(Some(base_mob))
        (Desc { name: "snake".to_string(), icon: 71, color: color::GREEN })
        (Stats { power: 1, intrinsics: 0 })
        (Spawn { biome: Biome::Overland, commonness: 1000, min_depth: 1, category: Category::Mob })
        ;

    Prototype::new(Some(base_mob))
        (Desc { name: "ooze".to_string(), icon: 77, color: color::LIGHTSEAGREEN })
        (Stats { power: 3, intrinsics: 0 })
        (Spawn { biome: Biome::Dungeon, commonness: 1000, min_depth: 3, category: Category::Mob })
        ;
    // TODO: More mob types

    // Items
    Prototype::new(None)
        (Desc { name: "heart".to_string(), icon: 89, color: color::RED })
        (Spawn { biome: Biome::Anywhere, commonness: 1000, min_depth: 1, category: Category::Consumable })
        (Item { power: 2, item_type: ItemType::Instant, ability: Ability::HealInstant(2) })
        ;

    action::start_level(1);
}

// Components stuff ////////////////////////////////////////////////////

#[deriving(RustcEncodable, RustcDecodable)]
struct Comps {
    descs: VecMap<Desc>,
    map_memories: VecMap<MapMemory>,
    stats: VecMap<Stats>,
    spawns: VecMap<Spawn>,
    healths: VecMap<Health>,
    brains: VecMap<Brain>,
    items: VecMap<Item>,
    stats_caches: VecMap<StatsCache>,
}

impl Comps {
    pub fn new() -> Comps {
        Comps {
            descs: VecMap::new(),
            map_memories: VecMap::new(),
            stats: VecMap::new(),
            spawns: VecMap::new(),
            healths: VecMap::new(),
            brains: VecMap::new(),
            items: VecMap::new(),
            stats_caches: VecMap::new(),
        }
    }
}

macro_rules! comp_api {
    // Rust macros can't concatenate "_mut" to $name because reasons, so the
    // _mut suffix name needs to be passed in explicitly.
    { $name:ident, $name_mut:ident, $typ:ty } => {
        impl<'a> WorldState {
        pub fn $name(&'a self) -> ComponentRef<'a, $typ> {
            ComponentRef::new(&self.ecs, &self.comps.$name)
        }
        pub fn $name_mut(&'a mut self) -> ComponentRefMut<'a, $typ> {
            ComponentRefMut::new(&mut self.ecs, &mut self.comps.$name)
        }
        }
    }
}

// COMPONENTS CHECKPOINT
comp_api!(descs, descs_mut, Desc);
comp_api!(map_memories, map_memories_mut, MapMemory);
comp_api!(stats, stats_mut, Stats);
comp_api!(spawns, spawns_mut, Spawn);
comp_api!(healths, healths_mut, Health);
comp_api!(brains, brains_mut, Brain);
comp_api!(items, items_mut, Item);
comp_api!(stats_caches, stats_caches_mut, StatsCache);

/// Immutable component access.
pub struct ComponentRef<'a, C: 'static> {
    ecs: &'a Ecs,
    data: &'a VecMap<C>,
}

impl<'a, C> ComponentRef<'a, C> {
    fn new(ecs: &'a Ecs, data: &'a VecMap<C>) -> ComponentRef<'a, C> {
        ComponentRef {
            ecs: ecs,
            data: data,
        }
    }

    /// Fetch a component from given entity or its parent.
    pub fn get(self, Entity(idx): Entity) -> Option<&'a C> {
        match find_parent(|Entity(idx)| self.data.contains_key(&idx), self.ecs, Entity(idx)) {
            None => { None }
            Some(Entity(idx)) => { self.data.get(&idx) }
        }
    }

    /// Fetch a component from given entity. Do not search parent entities.
    pub fn get_local(self, Entity(idx): Entity) -> Option<&'a C> {
        self.data.get(&idx)
    }
}

/// Mutable component access.
pub struct ComponentRefMut<'a, C: 'static> {
    ecs: &'a mut Ecs,
    data: &'a mut VecMap<C>,
}

impl<'a, C: Clone> ComponentRefMut<'a, C> {
    fn new(ecs: &'a mut Ecs, data: &'a mut VecMap<C>) -> ComponentRefMut<'a, C> {
        ComponentRefMut {
            ecs: ecs,
            data: data,
        }
    }

    /// Fetch a component from given entity or its parent. Copy-on-write
    /// semantics: A component found on a parent entity will be copied on
    /// local entity for mutation.
    pub fn get(self, Entity(idx): Entity) -> Option<&'a mut C> {
        match find_parent(|Entity(idx)| self.data.contains_key(&idx), self.ecs, Entity(idx)) {
            None => { None }
            Some(Entity(idx2)) if idx2 == idx => { self.data.get_mut(&idx2) }
            Some(Entity(idx2)) => {
                // Copy-on-write: Make a local copy of inherited component
                // when asking for mutable access.
                let cow = self.data.get(&idx2).expect("parent component lost").clone();
                self.data.insert(idx, cow);
                self.data.get_mut(&idx)
            }
        }
    }

    /// Add a component to entity.
    pub fn insert(self, Entity(idx): Entity, comp: C) {
        self.data.insert(idx, comp);
    }

    /// Remove a component from an entity. This will make a parent entity's
    /// component visible instead, if there is one. There is currently no way
    /// to hide components present in a parent entity.
    pub fn remove(self, Entity(idx): Entity) {
        self.data.remove(&idx);
    }
}

fn find_parent<P: Fn<(Entity,), bool>>(p: P, ecs: &Ecs, e: Entity) -> Option<Entity> {
    let mut current = e;
    loop {
        if p(current) { return Some(current); }
        match ecs.parent(current) {
            Some(parent) => { current = parent; }
            None => { return None; }
        }
    }
}
