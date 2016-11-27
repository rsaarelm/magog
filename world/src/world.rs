use std::sync::Arc;
use std::io::{Read, Write};
use std::collections::HashMap;
use bincode::{self, serde};
use euclid::Point2D;
use calx_resource::ResourceStore;
use calx_ecs::Entity;
use calx_grid::Dir6;
use field::Field;
use spatial::{Place, Spatial};
use flags::Flags;
use location::{Location, Portal};
use components;
use stats;
use terrain;
use query::Query;
use command::{Command, CommandResult};
use mutate::Mutate;
use terraform::{Terraform, TerrainQuery};

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
#[derive(RustcEncodable, RustcDecodable)]
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
        let mut ret = World {
            version: GAME_VERSION.to_string(),
            ecs: Ecs::new(),
            terrain: Field::new(0),
            portals: HashMap::new(),
            spatial: Spatial::new(),
            flags: Flags::new(seed),
        };

        ret.init_level(1);

        ret
    }

    /* TODO: Reactivate for rustc-serialize

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
    */

    fn default_terrain_id(&self, loc: Location) -> u8 {
        terrain::Id::Rock as u8
    }
}

impl TerrainQuery for World {
    fn is_valid_location(&self, loc: Location) -> bool {
        ::on_screen(Point2D::new(loc.x as i32, loc.y as i32))
    }

    fn terrain(&self, loc: Location) -> Arc<terrain::Tile> {
        let mut idx = self.terrain.get(loc);

        if idx == 0 {
            idx = self.default_terrain_id(loc);
        }

        if idx == terrain::Id::Door as u8 {
            // Standing in the doorway opens the door.
            if self.has_mobs(loc) {
                idx = terrain::Id::OpenDoor as u8;
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

    fn is_untouched(&self, loc: Location) -> bool {
        !self.terrain.overrides(loc)
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

    fn tick(&self) -> u64 { self.flags.tick }

    fn rng_seed(&self) -> u32 { self.flags.seed }

    fn entities_at(&self, loc: Location) -> Vec<Entity> { self.spatial.entities_at(loc) }

    fn ecs<'a>(&'a self) -> &'a Ecs { &self.ecs }
}

impl Mutate for World {
    fn next_tick(&mut self) -> CommandResult {
        // TODO: Run AI
        // TODO: Clean dead
        self.flags.tick += 1;
        Ok(())
    }

    fn set_entity_location(&mut self, e: Entity, loc: Location) { self.spatial.insert_at(e, loc); }

    fn set_player(&mut self, player: Entity) { self.flags.player = Some(player); }

    fn spawn(&mut self, loadout: &Loadout) -> Entity { loadout.make(&mut self.ecs) }
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
    fn set_terrain(&mut self, loc: Location, terrain: u8) {
        self.terrain.set(loc, terrain);
    }

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
