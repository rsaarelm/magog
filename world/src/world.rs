use std::collections::VecMap;
use std::cell::RefCell;
use std::rand;
use serialize::json;
use rand::Rng;
use ecs::Ecs;
use area::Area;
use spatial::Spatial;
use flags::Flags;
use mob::Mob;
use entity::Entity;
use action;
use {EntityKind};
use map_memory::MapMemory;
use desc::Desc;

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

#[deriving(Encodable, Decodable)]
struct Comps {
    mob: VecMap<Mob>,
    kind: VecMap<EntityKind>,
    map_memory: VecMap<MapMemory>,
    desc: VecMap<Desc>,
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
            comps: Comps {
                mob: VecMap::new(),
                kind: VecMap::new(),
                map_memory: VecMap::new(),
                desc: VecMap::new(),
            }
        }
    }

    // COMPONENTS CHECKPOINT
    // XXX: Boilerplate
    pub fn kinds(&'a self) ->                    ComponentRef<'a, EntityKind>            { ComponentRef::new(&self.ecs, &self.comps.kind) }
    pub fn kinds_mut(&'a mut self) ->         ComponentRefMut<'a, EntityKind> { ComponentRefMut::new(&mut self.ecs, &mut self.comps.kind) }
    pub fn mobs(&'a self) ->                     ComponentRef<'a, Mob>                   { ComponentRef::new(&self.ecs, &self.comps.mob) }
    pub fn mobs_mut(&'a mut self) ->          ComponentRefMut<'a, Mob>        { ComponentRefMut::new(&mut self.ecs, &mut self.comps.mob) }
    pub fn map_memories(&'a self) ->             ComponentRef<'a, MapMemory>             { ComponentRef::new(&self.ecs, &self.comps.map_memory) }
    pub fn map_memories_mut(&'a mut self) ->  ComponentRefMut<'a, MapMemory>  { ComponentRefMut::new(&mut self.ecs, &mut self.comps.map_memory) }
    pub fn descs(&'a self) ->                    ComponentRef<'a, Desc>                  { ComponentRef::new(&self.ecs, &self.comps.desc) }
    pub fn descs_mut(&'a mut self) ->         ComponentRefMut<'a, Desc>       { ComponentRefMut::new(&mut self.ecs, &mut self.comps.desc) }
}

/// Set up a fresh start-game world state with an optional fixed random number
/// generator seed. Calling init_world will cause any existing world state to
/// be discarded.
pub fn init_world(seed: Option<u32>) {
    WORLD_STATE.with(|w| *w.borrow_mut() = WorldState::new(seed));
    action::start_level(1);
}

#[deriving(Copy)]
pub struct ComponentRef<'a, C: 'static> {
    ecs: &'a Ecs,
    data: &'a VecMap<C>,
}

impl<'a, C> ComponentRef<'a, C> {
    pub fn new(ecs: &'a Ecs, data: &'a VecMap<C>) -> ComponentRef<'a, C> {
        ComponentRef {
            ecs: ecs,
            data: data,
        }
    }

    pub fn get(self, Entity(idx): Entity) -> Option<&'a C> {
        match find_parent(|Entity(idx)| self.data.contains_key(&idx), self.ecs, Entity(idx)) {
            None => { None }
            Some(Entity(idx)) => { self.data.get(&idx) }
        }
    }
}

pub struct ComponentRefMut<'a, C: 'static> {
    ecs: &'a mut Ecs,
    data: &'a mut VecMap<C>,
}

impl<'a, C: Clone> ComponentRefMut<'a, C> {
    pub fn new(ecs: &'a mut Ecs, data: &'a mut VecMap<C>) -> ComponentRefMut<'a, C> {
        ComponentRefMut {
            ecs: ecs,
            data: data,
        }
    }

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

    pub fn insert(self, Entity(idx): Entity, comp: C) {
        self.data.insert(idx, comp);
    }

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
