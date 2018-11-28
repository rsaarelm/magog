use crate::command::{Command, CommandResult};
use crate::components;
use crate::event::Event;
use crate::flags::Flags;
use crate::fov::SightFov;
use crate::item::Slot;
use crate::location::{Location, Portal};
use crate::mutate::Mutate;
use crate::query::Query;
use crate::spatial::{Place, Spatial};
use crate::terraform::{Terraform, TerrainQuery};
use crate::terrain::Terrain;
use crate::volume::Volume;
use crate::worldgen::Worldgen;
use crate::Rng;
use calx::{seeded_rng, HexFov, HexFovIter};
use calx_ecs::Entity;
use ron;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashSet;
use std::error::Error;
use std::io::{Read, Write};
use std::iter::FromIterator;
use std::slice;

pub const GAME_VERSION: &str = "0.1.0";

calx_ecs::build_ecs! {
    desc: components::Desc,
    anim: components::Anim,
    map_memory: components::MapMemory,
    health: components::Health,
    brain: components::Brain,
    item: components::Item,
    stats: components::StatsComponent,
    status: components::Statuses,
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
    worldgen: Worldgen,
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
            worldgen: Worldgen::new(seed),
            spatial: Spatial::new(),
            flags: Flags::new(),
            rng: seeded_rng(&seed),
            events: Vec::new(),
        };

        // XXX: Clone to not run into borrow checker...
        for (loc, spawn) in ret
            .worldgen
            .spawns()
            .cloned()
            .collect::<Vec<(Location, Loadout)>>()
        {
            ret.spawn(&spawn, loc);
        }

        // TODO non-lexical borrow
        let player_entry = ret.worldgen.player_entry();
        ret.spawn_player(player_entry);

        ret
    }

    pub fn load<R: Read>(reader: &mut R) -> Result<World, Box<dyn Error>> {
        let ret: ron::de::Result<World> = ron::de::from_reader(reader);
        if let Ok(ref x) = ret {
            if x.version != GAME_VERSION {
                panic!(
                    "Save game version {} does not match current version {}",
                    x.version, GAME_VERSION
                );
            }
        }
        Ok(ret?)
    }

    pub fn save<W: Write>(&self, writer: &mut W) -> Result<(), Box<dyn Error>> {
        let enc = ron::ser::to_string_pretty(self, Default::default())?;
        // TODO: Handle error from writer too...
        writeln!(writer, "{}", enc)?;
        Ok(())
    }
}

impl TerrainQuery for World {
    fn is_valid_location(&self, _loc: Location) -> bool {
        //Location::origin().v2_at(loc).map_or(false, ::on_screen)
        true
    }

    fn terrain(&self, loc: Location) -> Terrain {
        let mut t = self.worldgen.get_terrain(loc);

        if t == Terrain::Door && self.has_mobs(loc) {
            // Standing in the doorway opens the door.
            t = Terrain::OpenDoor;
        }

        t
    }

    fn portal(&self, loc: Location) -> Option<Location> { self.worldgen.get_portal(loc) }

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

    fn get_tick(&self) -> u64 { self.flags.tick }

    fn rng_seed(&self) -> u32 { self.worldgen.seed() }

    fn entities(&self) -> slice::Iter<'_, Entity> { self.ecs.iter() }

    fn entities_at(&self, loc: Location) -> Vec<Entity> { self.spatial.entities_at(loc) }

    fn entities_in(&self, parent: Entity) -> Vec<Entity> { self.spatial.entities_in(parent) }

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

        self.tick_anims();

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
        self.spatial.equip(e, parent, slot);
        self.rebuild_stats(parent);
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

        if let Some(origin) = self.location(e) {
            const DEFAULT_FOV_RANGE: u32 = 7;
            const OVERLAND_FOV_RANGE: u32 = 40;

            // Long-range sight while in overworld.
            let range = if origin.z == 0 {
                OVERLAND_FOV_RANGE
            } else {
                DEFAULT_FOV_RANGE
            };

            let fov: HashSet<Location> = HashSet::from_iter(
                HexFov::new(SightFov::new(self, range, origin))
                    .add_fake_isometric_acute_corners(|pos, a| {
                        self.terrain(a.origin + pos).is_wall()
                    })
                    .map(|(pos, a)| a.origin + pos),
            );

            let memory = &mut self.ecs.map_memory[e];
            memory.seen.clear();

            for &loc in &fov {
                memory.seen.insert(loc);
                // Don't spread map memory to other sectors we haven't "properly" seen yet.
                // The edges are still shown in visual FOV for display niceness.
                if loc.sector() == origin.sector() {
                    memory.remembered.insert(loc);
                }
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
