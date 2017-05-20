use std::io::{Read, Write};
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::slice;
use bincode;
use calx_ecs::Entity;
use calx_grid::{Dir6, HexFov};
use field::Field;
use spatial::{Place, Spatial};
use flags::Flags;
use location::{Location, Portal};
use components;
use stats;
use terrain::Terrain;
use query::Query;
use command::{Command, CommandResult};
use mutate::Mutate;
use terraform::{Terraform, TerrainQuery};
use fov::SightFov;

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
    /// Terrain data.
    terrain: Field<Terrain>,
    /// Optional portals between map zones.
    portals: HashMap<Location, Portal>,
    /// Spatial index for game entities.
    spatial: Spatial,
    /// Global gamestate flags.
    flags: Flags,
}

impl<'a> World {
    pub fn new(seed: u32) -> World {
        World {
            version: GAME_VERSION.to_string(),
            ecs: Ecs::new(),
            terrain: Field::new(Terrain::Empty),
            portals: HashMap::new(),
            spatial: Spatial::new(),
            flags: Flags::new(seed),
        }
    }

    pub fn load<R: Read>(reader: &mut R) -> bincode::Result<World> {
        let ret: bincode::Result<World> =
            bincode::deserialize_from(reader, bincode::Infinite);
        if let Ok(ref x) = ret {
            if x.version != GAME_VERSION {
                panic!("Save game version {} does not match current version {}",
                       x.version,
                       GAME_VERSION);
            }
        }
        ret
    }

    pub fn save<W: Write>(&self, writer: &mut W) -> bincode::Result<()> {
        bincode::serialize_into(writer, self, bincode::Infinite)
    }
}

impl TerrainQuery for World {
    fn is_valid_location(&self, loc: Location) -> bool {
        Location::origin().v2_at(loc).map_or(false, ::on_screen)
    }

    fn terrain(&self, loc: Location) -> Terrain {
        let mut t = self.terrain.get(loc);

        if t == Terrain::Door && self.has_mobs(loc) {
            // Standing in the doorway opens the door.
            t = Terrain::OpenDoor;
        }

        t
    }

    fn portal(&self, loc: Location) -> Option<Location> { self.portals.get(&loc).map(|&p| loc + p) }

    fn is_untouched(&self, loc: Location) -> bool { !self.terrain.overrides(loc) }
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

    fn rng_seed(&self) -> u32 { self.flags.seed }

    fn entities(&self) -> slice::Iter<Entity> { self.ecs.iter() }

    fn entities_at(&self, loc: Location) -> Vec<Entity> { self.spatial.entities_at(loc) }

    fn ecs(&self) -> &Ecs { &self.ecs }
}

impl Mutate for World {
    fn next_tick(&mut self) -> CommandResult {
        // TODO: Run AI
        self.clean_dead();
        self.flags.tick += 1;
        Ok(())
    }

    fn set_entity_location(&mut self, e: Entity, loc: Location) { self.spatial.insert_at(e, loc); }

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

            let fov: HashSet<Location> =
                HashSet::from_iter(HexFov::new(SightFov::new(self, DEFAULT_FOV_RANGE, loc))
                                       .map(|(pos, a)| a.origin + pos));

            let memory = &mut self.ecs.map_memory[e];
            memory.seen.clear();

            for &loc in &fov {
                memory.seen.insert(loc);
                memory.remembered.insert(loc);
            }
        }
    }
}

impl Command for World {
    fn step(&mut self, dir: Dir6) -> CommandResult {
        let player = try!(self.player().ok_or(()));
        try!(self.entity_step(player, dir));
        self.next_tick()
    }

    fn melee(&mut self, dir: Dir6) -> CommandResult {
        let player = try!(self.player().ok_or(()));
        try!(self.entity_melee(player, dir));
        self.next_tick()
    }

    fn pass(&mut self) -> CommandResult { self.next_tick() }
}

impl Terraform for World {
    fn set_terrain(&mut self, loc: Location, terrain: Terrain) { self.terrain.set(loc, terrain); }

    fn set_portal(&mut self, loc: Location, mut portal: Portal) {
        let target_loc = loc + portal;
        // Don't create portal chains, if the target cell has another portal, just direct to its
        // destination.
        if let Some(&p) = self.portals.get(&target_loc) {
            portal = portal + p;
        }

        if portal.dx == 0 && portal.dy == 0 && portal.z == loc.z {
            self.portals.remove(&loc);
        } else {
            self.portals.insert(loc, portal);
        }
    }

    fn remove_portal(&mut self, loc: Location) { self.portals.remove(&loc); }
}
