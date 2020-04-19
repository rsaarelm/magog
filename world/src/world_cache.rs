use crate::{
    location::{Location, Portal},
    map::MapCell,
    sector::{self, Sector, WorldSkeleton},
    spec::EntitySpawn,
    terrain::Terrain,
};
use euclid::{vec2, vec3};
use log::info;
use serde;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::mem;

/// Convention for maps, player always starts at origin sector.
pub const PLAYER_START_SECTOR: Sector = Sector::new(0, 0, 0);

/// Lazy instantiator for the generated world defined by random seed and skeleton.
///
/// Uses interior mutability to update the cache. Probably very thread unsafe.
pub struct WorldCache {
    seed: u32,
    skeleton: WorldSkeleton,

    internal_cache: RefCell<InternalCache>,
}

impl WorldCache {
    /// Initiate the cache given the world description.
    pub fn new(seed: u32, skeleton: WorldSkeleton) -> WorldCache {
        WorldCache {
            seed,
            skeleton,
            internal_cache: Default::default(),
        }
    }

    pub fn seed(&self) -> u32 { self.seed }

    /// Get the location where the player enters the world.
    pub fn player_entrance(&self) -> Location {
        // Player start in sector 0. Expect generation logic to set player position when
        // constructing the sector.
        self.generate(PLAYER_START_SECTOR);
        self.internal_cache.borrow().player_entrance
    }

    pub fn get_terrain(&self, loc: Location) -> Terrain {
        const FALLBACK_TERRAIN: Terrain = Terrain::Rock;

        self.finalize(Sector::from(loc));
        if let Some(t) = self.internal_cache.borrow().terrain.get(&loc).cloned() {
            t
        } else {
            FALLBACK_TERRAIN
        }
    }

    pub fn get_portal(&self, loc: Location) -> Option<Location> {
        self.finalize(Sector::from(loc));
        self.internal_cache
            .borrow()
            .portals
            .get(&loc)
            .map(|&p| loc + p)
    }

    pub fn sector_exists(&self, sector: Sector) -> bool { self.skeleton.contains_key(&sector) }

    /// Return latest list of spawns.
    ///
    /// `WorldCache` will return spawns from regions that have been loaded into cache. Caller will
    /// need to frequenty re-call spawns() method to see if new entities should be spawned. Caller
    /// is expected to keep track of which spawn items it has already seen and not re-instantiate
    /// them if the cache get regenerated.
    pub fn drain_spawns(&mut self) -> Vec<(Location, EntitySpawn)> {
        mem::replace(
            &mut self.internal_cache.borrow_mut().spawn_queue,
            Vec::new(),
        )
    }

    fn generate(&self, sector: Sector) {
        if !self.skeleton.contains_key(&sector) {
            // Outside world, skip.
            return;
        }

        if self
            .internal_cache
            .borrow()
            .constructed_sectors
            .contains(&sector)
        {
            // Already constructed, skip.
            return;
        }
        info!("WorldCache generating sector {:?}", sector);

        let map = sector::generate(self.seed, sector, &self.skeleton);

        // Load generated map into cache
        for (
            vec,
            MapCell {
                terrain, spawns, ..
            },
        ) in &map
        {
            let loc = Location::from(sector) + *vec;

            if *terrain != Terrain::default() {
                self.internal_cache
                    .borrow_mut()
                    .terrain
                    .insert(loc, *terrain);
            }

            // World cache uses (location, spawn string) as the key to see if it already has
            // created a spawn. Duplicate spawn strings from one location will be ignored after the
            // first one when mapgen wanted to create multiple entities.
            debug_assert!(
                spawns.iter().collect::<HashSet<_>>().len() == spawns.len(),
                "Duplicates in mapgen spawns will not be generated correctly"
            );

            // Put spawns in pending list, don't want them to go live yet because they'd trigger a
            // cache cascade once their AI starts running.
            for s in spawns {
                self.internal_cache
                    .borrow_mut()
                    .pending_spawns
                    .entry(sector)
                    .or_insert_with(Vec::new)
                    .push((loc, s.clone()));
            }
        }

        if sector == PLAYER_START_SECTOR {
            self.internal_cache.borrow_mut().player_entrance =
                Location::from(sector) + map.player_entrance();
        }

        self.internal_cache
            .borrow_mut()
            .constructed_sectors
            .insert(sector);
    }

    /// Finalize a sector and make it ready for play.
    ///
    /// This step generates stairway exits.
    fn finalize(&self, sector: Sector) {
        if !self.skeleton.contains_key(&sector) {
            return;
        }

        if self
            .internal_cache
            .borrow()
            .finalized_sectors
            .contains(&sector)
        {
            return;
        }

        info!("WorldCache finalizing sector {:?}", sector);

        // Make sure sectors above and below are generated so we can determine portal locations.
        let above = sector + vec3(0, 0, 1);
        let below = sector + vec3(0, 0, -1);
        self.generate(above);
        self.generate(below);
        self.generate(sector);

        if let Some(my_up) = self.upstairs(sector) {
            let their_down = self
                .downstairs(above)
                .expect("No matching downstairs found");
            self.make_stairs(their_down, my_up);
        }

        if let Some(my_down) = self.downstairs(sector) {
            let their_up = self.upstairs(below).expect("No matching upstairs found");
            self.make_stairs(my_down, their_up);
        }

        let pending_spawns = self
            .internal_cache
            .borrow_mut()
            .pending_spawns
            .remove(&sector);

        if let Some(mut pending_spawns) = pending_spawns {
            self.internal_cache
                .borrow_mut()
                .spawn_queue
                .append(&mut pending_spawns);
        }

        self.internal_cache
            .borrow_mut()
            .finalized_sectors
            .insert(sector);
    }

    /// Find location of stairs down on sector.
    fn downstairs(&self, sector: Sector) -> Option<Location> {
        self.generate(sector);
        for loc in sector.iter() {
            if let Some(Terrain::Downstairs) = self.internal_cache.borrow().terrain.get(&loc) {
                return Some(loc);
            }
        }
        None
    }

    fn upstairs(&self, sector: Sector) -> Option<Location> {
        self.generate(sector);
        for loc in sector.iter() {
            if let Some(Terrain::Upstairs) = self.internal_cache.borrow().terrain.get(&loc) {
                return Some(loc);
            }
        }
        None
    }

    /// Make a two-way stairwell portal.
    fn make_stairs(&self, downstairs: Location, upstairs: Location) {
        self.portal(upstairs, downstairs - vec2(1, 1));
        self.portal(downstairs, upstairs + vec2(1, 1));
    }

    /// Punch a (one-way) portal between two points.
    fn portal(&self, origin: Location, destination: Location) {
        self.internal_cache
            .borrow_mut()
            .portals
            .insert(origin, Portal::new(origin, destination));
    }
}

impl serde::Serialize for WorldCache {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        (self.seed, &self.skeleton).serialize(s)
    }
}

impl<'a> serde::Deserialize<'a> for WorldCache {
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        let (seed, skeleton) = serde::Deserialize::deserialize(d)?;
        Ok(WorldCache::new(seed, skeleton))
    }
}

#[derive(Default)]
struct InternalCache {
    /// Sectors for which the terrain and entities have been constructed
    constructed_sectors: HashSet<Sector>,

    /// Sectors for which portal data has been generated.
    ///
    /// Portal data requires that sectors above and below have been constructed to see the opposite
    /// end of a stairway, so there needs to be the separate construction and finalization steps.
    finalized_sectors: HashSet<Sector>,

    terrain: HashMap<Location, Terrain>,
    portals: HashMap<Location, Portal>,

    pending_spawns: HashMap<Sector, Vec<(Location, EntitySpawn)>>,
    spawn_queue: Vec<(Location, EntitySpawn)>,

    player_entrance: Location,
}
