use Rng;
use calx_ecs::Entity;
use rand::SeedableRng;
use calx_grid::HexFov;
use command::{Command, CommandResult};
use components;
use errors::*;
use event::Event;
use flags::Flags;
use fov::SightFov;
use item::Slot;
use location::{Location, Portal};
use mutate::Mutate;
use query::Query;
use ron;
use spatial::{Place, Spatial};
use stats;
use std::collections::HashSet;
use std::io::{Read, Write};
use std::iter::FromIterator;
use std::slice;
use terraform::{Terraform, TerrainQuery};
use terrain::Terrain;
use volume::Volume;
use world_gen::WorldGen;

pub const GAME_VERSION: &'static str = "0.1.0";

Ecs! {
    desc: components::Desc,
    map_memory: components::MapMemory,
    health: components::Health,
    brain: components::Brain,
    item: components::Item,
    composite_stats: components::CompositeStats,
    stats: stats::Stats,
}

/// Toplevel game state object.
#[derive(Serialize, Deserialize)]
pub struct World {
    /// Game version. Not mutable in the slightest, but the simplest way to
    /// get versioned save files is to just drop it here.
    version: String,
    /// Entity component system.
    ecs: Ecs,
    /// Static startup game world
    world_gen: WorldGen,
    /// Spatial index for game entities.
    spatial: Spatial,
    /// Global gamestate flags.
    flags: Flags,
    /// Persistent random number generator.
    rng: Rng,
    /// Event queue
    events: Vec<Event>,
}

impl<'a> World {
    pub fn new(seed: u32) -> World {
        let mut ret = World {
            version: GAME_VERSION.to_string(),
            ecs: Ecs::new(),
            world_gen: WorldGen::new(seed),
            spatial: Spatial::new(),
            flags: Flags::new(),
            rng: SeedableRng::from_seed([seed, seed, seed, seed]),
            events: Vec::new(),
        };

        // XXX: Clone to not run into borrow checker...
        for (loc, spawn) in ret.world_gen
            .spawns()
            .cloned()
            .collect::<Vec<(Location, Loadout)>>()
            .into_iter()
        {
            ret.spawn(&spawn, loc);
        }

        let player_entry = ret.world_gen.player_entry();
        ret.spawn_player(player_entry);

        ret
    }

    pub fn load<R: Read>(reader: &mut R) -> Result<World> {
        let ret: ron::de::Result<World> = ron::de::from_reader(reader);
        if let Ok(ref x) = ret {
            if x.version != GAME_VERSION {
                panic!(
                    "Save game version {} does not match current version {}",
                    x.version,
                    GAME_VERSION
                );
            }
        }
        Ok(ret?)
    }

    pub fn save<W: Write>(&self, writer: &mut W) -> Result<()> {
        let enc = ron::ser::to_string(self)?;
        // TODO: Handle error from writer too...
        writeln!(writer, "{}", enc)?;
        Ok(())
    }
}

impl TerrainQuery for World {
    fn is_valid_location(&self, loc: Location) -> bool {
        Location::origin().v2_at(loc).map_or(false, ::on_screen)
    }

    fn terrain(&self, loc: Location) -> Terrain {
        let mut t = self.world_gen.get_terrain(loc);

        if t == Terrain::Door && self.has_mobs(loc) {
            // Standing in the doorway opens the door.
            t = Terrain::OpenDoor;
        }

        t
    }

    fn portal(&self, loc: Location) -> Option<Location> { self.world_gen.get_portal(loc) }

    fn is_untouched(&self, _loc: Location) -> bool { unimplemented!() }
}

impl Query for World {
    fn location(&self, e: Entity) -> Option<Location> {
        match self.spatial.get(e) {
            Some(Place::At(loc)) => Some(loc),
            Some(Place::In(container, _)) => self.location(container),
            _ => None,
        }
    }

    fn player(&self) -> Option<Entity> {
        if let Some(p) = self.flags.player {
            if self.is_alive(p) {
                return Some(p);
            }
        }

        None
    }

    fn tick(&self) -> u64 { self.flags.tick }

    fn rng_seed(&self) -> u32 { self.world_gen.seed() }

    fn entities(&self) -> slice::Iter<Entity> { self.ecs.iter() }

    fn entities_at(&self, loc: Location) -> Vec<Entity> { self.spatial.entities_at(loc) }

    fn ecs(&self) -> &Ecs { &self.ecs }

    fn entity_equipped(&self, parent: Entity, slot: Slot) -> Option<Entity> {
        self.spatial.entity_equipped(parent, slot)
    }

    fn entity_contains(&self, parent: Entity, child: Entity) -> bool {
        self.spatial.contains(parent, child)
    }

    fn sphere_volume(&self, origin: Location, radius: u32) -> Volume {
        Volume::sphere(self, origin, radius)
    }
}

impl Mutate for World {
    fn next_tick(&mut self) -> CommandResult {
        use std::mem;

        self.ai_main();

        self.clean_dead();
        self.flags.tick += 1;

        // Dump events.
        let mut events = Vec::new();
        mem::swap(&mut self.events, &mut events);
        Ok(events)
    }

    fn set_entity_location(&mut self, e: Entity, loc: Location) { self.spatial.insert_at(e, loc); }

    fn equip_item(&mut self, e: Entity, parent: Entity, slot: Slot) {
        self.spatial.equip(e, parent, slot)
    }

    fn set_player(&mut self, player: Option<Entity>) { self.flags.player = player; }

    fn spawn(&mut self, loadout: &Loadout, loc: Location) -> Entity {
        let e = loadout.make(&mut self.ecs);
        self.place_entity(e, loc);
        e
    }

    fn kill_entity(&mut self, e: Entity) { self.spatial.remove(e); }

    fn remove_entity(&mut self, e: Entity) { self.ecs.remove(e); }

    fn do_fov(&mut self, e: Entity) {
        if !self.ecs.map_memory.contains(e) {
            return;
        }

        if let Some(loc) = self.location(e) {
            const DEFAULT_FOV_RANGE: u32 = 12;

            let fov: HashSet<Location> = HashSet::from_iter(
                HexFov::new(SightFov::new(self, DEFAULT_FOV_RANGE, loc))
                    .map(|(pos, a)| a.origin + pos),
            );

            let memory = &mut self.ecs.map_memory[e];
            memory.seen.clear();

            for &loc in &fov {
                memory.seen.insert(loc);
                memory.remembered.insert(loc);
            }
        }
    }

    fn push_event(&mut self, event: Event) { self.events.push(event); }

    fn rng(&mut self) -> &mut Rng { &mut self.rng }

    fn ecs_mut(&mut self) -> &mut Ecs { &mut self.ecs }
}

impl Command for World {}

impl Terraform for World {
    fn set_terrain(&mut self, _loc: Location, _terrain: Terrain) {
        unimplemented!();
    }

    fn set_portal(&mut self, _loc: Location, _portal: Portal) {
        unimplemented!();
    }

    fn remove_portal(&mut self, _loc: Location) {
        unimplemented!();
    }
}
