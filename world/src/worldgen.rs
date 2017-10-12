use Prefab;
use calx_grid::{Dijkstra, Dir6};
use euclid::{vec2, Size2D};
use form::{self, Form};
use location::{Location, Portal, Sector};
use mapgen::{self, DigCavesGen};
use rand::{self, Rand, Rng, SeedableRng};
use serde;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::slice;
use terrain::Terrain;
use world::Loadout;

/// Static generated world.
pub struct Worldgen {
    seed: u32,
    terrain: HashMap<Location, Terrain>,
    portals: HashMap<Location, Portal>,
    spawns: Vec<(Location, Loadout)>,
    player_entry: Location,
}

impl Worldgen {
    pub fn new(seed: u32) -> Worldgen {
        let mut ret = Worldgen {
            seed: seed,
            terrain: HashMap::new(),
            portals: HashMap::new(),
            spawns: Vec::new(),
            player_entry: Location::new(0, 0, 0),
        };

        let mut rng: ::Rng = SeedableRng::from_seed([seed, seed, seed, seed]);

        let mut cave_entrance = Location::new(9, 0, 0);
        ret.cave_entrance(cave_entrance);
        ret.terrain.insert(
            cave_entrance + vec2(1, 1),
            Terrain::Gate,
        );

        for depth in 1..11 {
            let up_stairs;
            let down_stairs;
            let mut spawns = Vec::new();

            {
                let mut digger = SectorDigger::new(&mut ret, Sector::new(0, 0, depth as i16));
                let gen = DigCavesGen::new(digger.domain());
                gen.dig(&mut rng, &mut digger);

                up_stairs = digger.up_portal.expect("Mapgen didn't create stairs up");
                down_stairs = digger.down_portal.expect(
                    "Mapgen didn't create stairs down",
                );

                // Spawn
                digger.clear_spawns_near_entrance(12);

                let mut spawn_locs = rand::sample(&mut rng, digger.spawn_region.iter(), 20);
                let n_spawns = spawn_locs.len();

                let items = Form::filter(|f| f.is_item() && f.at_depth(depth));
                for &loc in spawn_locs.drain(0..n_spawns / 2) {
                    spawns.push((
                        loc,
                        form::rand(&mut rng, &items)
                            .expect("No item spawn")
                            .loadout
                            .clone(),
                    ))
                }

                let mobs = Form::filter(|f| f.is_mob() && f.at_depth(depth));
                for &loc in spawn_locs {
                    spawns.push((
                        loc,
                        form::rand(&mut rng, &mobs)
                            .expect("No mob spawn")
                            .loadout
                            .clone(),
                    ))
                }
            }

            ret.spawns.extend(spawns.into_iter());
            ret.portal(cave_entrance, up_stairs + vec2(1, 1));
            ret.portal(up_stairs, cave_entrance - vec2(1, 1));
            cave_entrance = down_stairs;

            // TODO: Generator needs an option to not generate stairs down on bottom level
        }

        ret
    }

    fn load_prefab(&mut self, origin: Location, prefab: &Prefab) {
        for (p, &(ref terrain, ref entities)) in prefab.iter() {
            let loc = origin + p;

            self.terrain.insert(loc, *terrain);

            for spawn in entities.iter() {
                if spawn == "player" {
                    self.player_entry = loc;
                } else {
                    let form = Form::named(spawn).expect(&format!(
                        "Bad prefab: Form '{}' not found!",
                        spawn
                    ));
                    self.spawns.push((loc, form.loadout.clone()));
                }
            }
        }
    }

    pub fn seed(&self) -> u32 { self.seed }

    pub fn get_terrain(&self, loc: Location) -> Terrain {
        if let Some(&t) = self.terrain.get(&loc) {
            t
        } else {
            self.default_terrain(loc)
        }
    }

    fn default_terrain(&self, loc: Location) -> Terrain {
        use Terrain::*;
        if loc.z == 0 {
            match loc.noise() {
                n if n > 0.8 => Tree,
                n if n > -0.8 => Grass,
                _ => Water,
            }
        } else {
            Terrain::Rock
        }
    }

    pub fn get_portal(&self, loc: Location) -> Option<Location> {
        self.portals.get(&loc).map(|&p| loc + p)
    }

    pub fn spawns(&self) -> slice::Iter<(Location, Loadout)> { self.spawns.iter() }

    pub fn player_entry(&self) -> Location { self.player_entry }

    /// Make a cave entrance going down.
    fn cave_entrance(&mut self, loc: Location) {
        const DOWNBOUND_ENCLOSURE: [(i32, i32); 5] = [(1, 0), (0, 1), (2, 1), (1, 2), (2, 2)];

        for &v in &DOWNBOUND_ENCLOSURE {
            let loc = loc + vec2(v.0, v.1);
            self.terrain.insert(loc, Terrain::Rock);
        }

        self.terrain.insert(loc, Terrain::Ground);
        self.terrain.insert(loc + vec2(1, 1), Terrain::Ground);
    }

    /// Punch a (one-way) portal between two points.
    fn portal(&mut self, origin: Location, destination: Location) {
        self.portals.insert(
            origin,
            Portal::new(origin, destination),
        );
    }
}

impl serde::Serialize for Worldgen {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.seed.serialize(s)
    }
}

impl<'a> serde::Deserialize<'a> for Worldgen {
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        Ok(Worldgen::new(serde::Deserialize::deserialize(d)?))
    }
}

struct SectorDigger<'a> {
    worldgen: &'a mut Worldgen,
    sector: Sector,
    up_portal: Option<Location>,
    down_portal: Option<Location>,
    spawn_region: BTreeSet<Location>,
}

impl<'a> SectorDigger<'a> {
    fn new(worldgen: &'a mut Worldgen, sector: Sector) -> SectorDigger<'a> {
        SectorDigger {
            worldgen,
            sector,
            up_portal: None,
            down_portal: None,
            spawn_region: BTreeSet::new(),
        }
    }

    // TODO return impl
    fn domain(&self) -> Vec<mapgen::Point2D> {
        fn is_next_to_diagonal_sector(loc: Location) -> bool {
            Dir6::iter()
                .map(|&d| (loc + d).sector().taxicab_distance(loc.sector()))
                .any(|d| d > 1)
        }

        let mut ret = Vec::new();

        let sector_origin = self.sector.origin();
        let mapgen_origin = mapgen::Point2D::zero();

        for loc in self.sector.iter() {
            let pos = mapgen_origin +
                sector_origin.v2_at(loc).expect(
                    "Sector points are not Euclidean",
                );

            // Okay, this part is a bit hairy, hang on.
            // Sectors are arranged in a rectangular grid, so there are some cells where you can
            // step diagonally across two sector boundaries. However, we want to pretend that
            // sectors are only connected in the four cardinal directions, so we omit these cells
            // from the domain to prevent surprise holes between two diagonally adjacent shafts.
            if is_next_to_diagonal_sector(loc) {
                continue;
            }

            ret.push(pos);
        }

        ret
    }

    fn loc(&self, pos: mapgen::Point2D) -> Location { self.sector.origin() + pos.to_vector() }

    fn set(&mut self, loc: Location, terrain: Terrain) {
        if terrain != Terrain::Gate {
            debug_assert_ne!(self.up_portal, Some(loc), "mapgen overwriting exit");
            debug_assert_ne!(self.down_portal, Some(loc), "mapgen overwriting exit");
        }
        self.worldgen.terrain.insert(loc, terrain);
    }

    fn clear_spawns_near_entrance(&mut self, range: u32) {
        for loc in Dijkstra::new(
            vec![self.up_portal.expect("No entrance generated")],
            |loc| {
                self.worldgen
                    .terrain
                    .get(loc)
                    .cloned()
                    .unwrap_or(Terrain::Empty)
                    .is_open()
            },
            10_000,
        ).weights
            .iter()
            .filter_map(|(loc, &w)| if w <= range { Some(loc) } else { None })
        {
            self.spawn_region.remove(&loc);
        }
    }
}

impl<'a> mapgen::Dungeon for SectorDigger<'a> {
    type Prefab = Room;

    /// Return a random prefab room.
    fn sample_prefab<R: Rng>(&mut self, rng: &mut R) -> Self::Prefab { rng.gen() }

    /// Add a large open continuous region to dungeon.
    fn dig_chamber<I: IntoIterator<Item = mapgen::Point2D>>(&mut self, area: I) {
        for pos in area.into_iter() {
            let loc = self.loc(pos);
            self.spawn_region.insert(loc);
            self.set(loc, Terrain::Ground);
        }
    }

    /// Add a narrow corridor to dungeon.
    fn dig_corridor<I: IntoIterator<Item = mapgen::Point2D>>(&mut self, path: I) {
        for pos in path.into_iter() {
            let loc = self.loc(pos);
            // Do not spawn things in corridors.
            self.spawn_region.remove(&loc);
            self.set(loc, Terrain::Ground);
        }
    }

    fn add_prefab(&mut self, prefab: &Self::Prefab, pos: mapgen::Point2D) {
        // TODO
        unimplemented!();
    }

    fn add_door(&mut self, pos: mapgen::Point2D) {
        let loc = self.loc(pos);
        self.spawn_region.remove(&loc);
        self.set(loc, Terrain::Door);
    }

    fn add_up_stairs(&mut self, pos: mapgen::Point2D) {
        let loc = self.loc(pos);
        self.up_portal = Some(loc);
        self.spawn_region.remove(&loc);
        // TODO: Add debug assertion that up-stairs are dug in the upper wall of a room
        self.set(loc, Terrain::Gate);
    }

    fn add_down_stairs(&mut self, pos: mapgen::Point2D) {
        let loc = self.loc(pos);
        self.down_portal = Some(loc);
        self.spawn_region.remove(&loc);
        self.set(loc, Terrain::Gate);

        // Visual hack to make the down-stairs show up better, carve out the rock blob that would
        // be drawn partially in front of it on screen.
        self.set(loc + vec2(1, 1), Terrain::Empty);
        // TODO: Add a debug assertion here to check that the exit location has proper enclosure to
        // begin with, both it and the rock cell in front of it need to be surrounded by undug
        // front tiles.
        self.spawn_region.remove(&(loc + vec2(1, 1))); // Just in case
    }
}

struct Room {
    size: Size2D<i32>,
}

impl mapgen::Prefab for Room {
    fn contains(&self, pos: mapgen::Point2D) -> bool {
        pos.x >= 0 && pos.y >= 0 && pos.x < self.size.width && pos.y < self.size.height
    }

    fn can_make_door(&self, pos: mapgen::Point2D) -> bool {
        let on_x_wall = pos.x == 0 || pos.x == self.size.width - 1;
        let on_y_wall = pos.y == 0 || pos.y == self.size.height - 1;

        // Must touch one wall, but touching both makes it a corner and we don't want rooms there.
        on_x_wall ^ on_y_wall
    }

    fn size(&self) -> mapgen::Size2D { self.size }
}

impl Rand for Room {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        Room { size: Size2D::new(rng.gen_range(3, 10), rng.gen_range(3, 10)) }
    }
}
