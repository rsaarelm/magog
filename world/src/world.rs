use std::rc::Rc;
use std::io::{Read, Write};
use std::collections::HashMap;
use bincode::{self, serde};
use calx_resource::{Resource, ResourceStore};
use calx_ecs::Entity;
use calx_grid::Dir6;
use field::Field;
use spatial::{Place, Spatial};
use flags::Flags;
use location::{Location, Portal};
use brush::Brush;
use components::{self, Alignment, BrainState};
use stats;
use terrain;
use FovStatus;
use query::Query;
use command::{Command, CommandResult};
use mutate::Mutate;
use terraform::Terraform;

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
    terrain: Field<u8>,
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
            terrain: Field::new(0),
            portals: HashMap::new(),
            spatial: Spatial::new(),
            flags: Flags::new(seed),
        }
    }

    pub fn load<R: Read>(reader: &mut R) -> serde::DeserializeResult<World> {
        let ret: serde::DeserializeResult<World> =
            serde::deserialize_from(reader, bincode::SizeLimit::Infinite);
        if let &Ok(ref x) = &ret {
            if &x.version != GAME_VERSION {
                panic!("Save game version {} does not match current version \
                        {}",
                       x.version,
                       GAME_VERSION);
            }
        }
        ret
    }

    pub fn save<W: Write>(&self, writer: &mut W) -> serde::SerializeResult<()> {
        serde::serialize_into(writer, self, bincode::SizeLimit::Infinite)
    }
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

    fn brain_state(&self, e: Entity) -> Option<BrainState> {
        self.ecs.brain.get(e).map_or(None, |brain| Some(brain.state))
    }

    fn tick(&self) -> u64 { self.flags.tick }

    fn is_mob(&self, e: Entity) -> bool { self.ecs.brain.contains(e) }

    fn alignment(&self, e: Entity) -> Option<Alignment> {
        self.ecs.brain.get(e).map(|b| b.alignment)
    }

    fn terrain(&self, loc: Location) -> Rc<terrain::Tile> {
        let mut idx = self.terrain.get(loc);

        if idx == 0 {
            use terrain::Id;
            // Empty terrain, inject custom stuff.
            match loc.noise() {
                x if x < 0.5 => idx = Id::Ground as u8,
                x if x < 0.75 => idx = Id::Grass as u8,
                x if x < 0.95 => idx = Id::Water as u8,
                _ => idx = Id::Tree as u8,
            }
        }

        terrain::Tile::get_resource(&idx).unwrap()

        // TODO: Add open/closed door mapping to terrain data, closed door terrain should have a field
        // that contains the terrain index of the corresponding open door tile.

        // TODO: Support terrain with brush variant distributions, like the grass case below that
        // occasionlly emits a fancier brush. The distribution needs to be embedded in the Tile struct.
        // The sampling needs loc noise, but is probably better done at the point where terrain is
        // being drawn than here, since we'll want to still have just one immutable terrain id
        // corresponding to all the variants.
        // Mobs standing on doors make the doors open.

        // if ret == TerrainType::Door && self.has_mobs(loc) {
        //     ret = TerrainType::OpenDoor;
        // }
        // // Grass is only occasionally fancy.
        // if ret == TerrainType::Grass {
        //     if loc.noise() > 0.85 {
        //         ret = TerrainType::Grass2;
        //     }
        // }
    }

    fn portal(&self, loc: Location) -> Option<Location> { self.portals.get(&loc).map(|&p| loc + p) }

    fn hp(&self, e: Entity) -> i32 {
        self.max_hp(e) - if self.ecs.health.contains(e) { self.ecs.health[e].wounds } else { 0 }
    }

    fn fov_status(&self, loc: Location) -> Option<FovStatus> {
        if let Some(p) = self.player() {
            if self.ecs.map_memory.contains(p) {
                if self.ecs.map_memory[p].seen.contains(&loc) {
                    return Some(FovStatus::Seen);
                }
                if self.ecs.map_memory[p].remembered.contains(&loc) {
                    return Some(FovStatus::Remembered);
                }
                return None;
            }
        }
        // Just show everything by default.
        Some(FovStatus::Seen)
    }

    fn entity_brush(&self, e: Entity) -> Option<Resource<Brush>> {
        self.ecs.desc.get(e).map(|x| x.brush.clone())
    }

    fn stats(&self, e: Entity) -> stats::Stats {
        self.ecs.composite_stats.get(e).map_or_else(|| self.base_stats(e), |x| x.0)
    }

    fn base_stats(&self, e: Entity) -> stats::Stats {
        self.ecs.stats.get(e).cloned().unwrap_or_default()
    }

    fn entities_at(&self, loc: Location) -> Vec<Entity> { self.spatial.entities_at(loc) }
}

impl Mutate for World {
    fn next_tick(&mut self) -> CommandResult {
        // TODO: Run AI
        // TODO: Clean dead
        self.flags.tick += 1;
        Ok(())
    }

    fn set_entity_location(&mut self, e: Entity, loc: Location) { self.spatial.insert_at(e, loc); }
}

impl Command for World {
    fn step(&mut self, dir: Dir6) -> CommandResult {
        if let Some(e) = self.player() {
            try!(self.entity_step(e, dir));
            self.next_tick()
        } else {
            Err(())
        }
    }

    fn melee(&mut self, dir: Dir6) -> CommandResult {
        if let Some(e) = self.player() {
            try!(self.entity_melee(e, dir));
            self.next_tick()
        } else {
            Err(())
        }
    }

    fn pass(&mut self) -> CommandResult { self.next_tick() }
}

impl Terraform for World {
    fn set_terrain(&mut self, loc: Location, terrain: u8) { self.terrain.set(loc, terrain); }

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
