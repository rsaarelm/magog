use std::collections::VecMap;
use std::cell::RefCell;
use std::rand;
use serialize::json;
use rand::Rng;
use calx::color;
use ecs::Ecs;
use area::Area;
use spatial::Spatial;
use flags::Flags;
use mob::{Mob, Intrinsic};
use entity::Entity;
use action;
use components::{Prototype};
use components::{Desc, Kind, MapMemory, MobStat, Spawn};
use {Biome};

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

    // Init the prototypes
    Prototype::new()
        (Desc { name: "Player".to_string(), icon: 51, color: color::RED })
        (MobStat { power: 6, intrinsics: Intrinsic::Hands as i32 })
        ;

    Prototype::new()
        (Desc { name: "Dreg".to_string(), icon: 72, color: color::OLIVE })
        (MobStat { power: 1, intrinsics: Intrinsic::Hands as i32 })
        (Spawn { biome: Biome::Anywhere, rarity: 10, min_depth: 1 })
        ;

    Prototype::new()
        (Desc { name: "Snake".to_string(), icon: 71, color: color::GREEN })
        (MobStat { power: 1, intrinsics: 0 })
        (Spawn { biome: Biome::Overland, rarity: 10, min_depth: 1 })
        ;

    Prototype::new()
        (Desc { name: "Ooze".to_string(), icon: 77, color: color::LIGHTSEAGREEN })
        (MobStat { power: 3, intrinsics: 0 })
        (Spawn { biome: Biome::Dungeon, rarity: 10, min_depth: 3 })
        ;

    action::start_level(1);
}

// Components stuff ////////////////////////////////////////////////////

#[deriving(Encodable, Decodable)]
struct Comps {
    descs: VecMap<Desc>,
    kinds: VecMap<Kind>,
    map_memories: VecMap<MapMemory>,
    mobs: VecMap<Mob>,
    mob_stats: VecMap<MobStat>,
    spawns: VecMap<Spawn>,
}

impl Comps {
    pub fn new() -> Comps {
        Comps {
            descs: VecMap::new(),
            kinds: VecMap::new(),
            map_memories: VecMap::new(),
            mobs: VecMap::new(),
            mob_stats: VecMap::new(),
            spawns: VecMap::new(),
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
comp_api!(descs, descs_mut, Desc)
comp_api!(kinds, kinds_mut, Kind)
comp_api!(map_memories, map_memories_mut, MapMemory)
comp_api!(mobs, mobs_mut, Mob)
comp_api!(mob_stats, mob_stats_mut, MobStat)
comp_api!(spawns, spawns_mut, Spawn)

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
                let cow = self.data.get(&idx2).unwrap().clone();
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
        if p(current) { return Some(e); }
        match ecs.parent(current) {
            Some(parent) => { current = parent; }
            None => { return None; }
        }
    }
}
