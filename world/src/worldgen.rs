use Prefab;
use calx_grid::{Dijkstra, Dir6};
use euclid::{vec2, Size2D};
use field::Field;
use form::{self, Form};
use location::{Location, Portal, Sector};
use mapgen::{self, DigCavesGen};
use rand::{self, Rand, Rng, SeedableRng};
use serde;
use std::cmp;
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
        ret.terrain.insert(cave_entrance, Terrain::Gate);

        for cave_z in 1..11 {
            let up_stairs;
            let down_stairs;

            {
                let mut digger = SectorDigger::new(&mut ret, Sector::new(0, 0, cave_z));
                let gen = DigCavesGen::new(digger.domain());
                gen.dig(&mut rng, &mut digger);

                up_stairs = digger.up_portal.expect("Mapgen didn't create stairs up");
                down_stairs = digger.down_portal.expect("Mapgen didn't create stairs down");
            }

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

    // TODO Move to mapgen
    #[deprecated]
    fn gen_caves<R: Rng>(&mut self, rng: &mut R, entrance: Location) -> Location {
        use self::Prototerrain::*;

        let mut cells_to_dig = 700;

        let mut map = screen_map(Location::new(0, 0, entrance.z));

        debug_assert_eq!(map.get(entrance), Unused);

        // Create portal enclosure
        entry_cave_enclosure(&mut map, entrance);

        let entrance = entrance + vec2(1, 1);

        let mut edge: BTreeSet<Location> = Dir6::iter()
            .map(|&d| entrance + vec2(2, 2) + d)
            .filter(|&loc| map.get(loc) == Unused)
            .collect();

        // Arbitrary long iteration, should break after digging a sufficient number of cells before
        // this.
        for _ in 0..10_000 {
            if edge.is_empty() {
                break;
            }

            let dig_loc = *rand::sample(rng, edge.iter(), 1)[0];

            // Prefer digging narrow corridors, there's an increasing chance to abort the dig when the
            // selected location is in a very open space.
            let adjacent_floors = Dir6::iter()
                .filter(|d| map.get(dig_loc + **d) == Floor)
                .count();
            if rng.gen_range(0, cmp::max(1, adjacent_floors * adjacent_floors)) != 0 {
                continue;
            }

            map.set(dig_loc, Floor);
            edge.extend(Dir6::iter().map(|&d| dig_loc + d).filter(|&loc| {
                map.get(loc) == Unused
            }));

            cells_to_dig -= 1;
            if cells_to_dig == 0 {
                break;
            }
        }

        // Postprocess
        let cells: Vec<_> = map.iter().map(|(&loc, &c)| (loc, c)).collect();
        for (loc, c) in cells {
            // Clear pillars
            if c == Unused {
                let adjacent_floors = Dir6::iter().filter(|d| map.get(loc + **d) == Floor).count();

                if adjacent_floors == 6 {
                    map.set(loc, Floor);
                }
            }
        }


        // Find opening for next map
        let openings: Vec<Location> = map.iter()
            .map(|(&loc, _)| loc)
            .filter(|&loc| can_be_path_down_opening(&map, entrance, loc))
            .collect();
        // XXX FIXME: There's actually no guarantees made that this can't fail
        // Fallback should be something like carving the exit to the bottom edge of the map
        assert!(!openings.is_empty(), "No exit candidates");
        let exit_loc = rand::sample(rng, openings, 1)[0];
        map.set(exit_loc, Border);


        // Spawns
        const MIN_DISTANCE_FROM_ENTRANCE: u32 = 10;
        let depth = entrance.z as i32;
        // Flood-fill the new map
        let mut spawn_map = Dijkstra::new(vec![entrance], |&loc| map.get(loc) == Floor, 10_000)
            .weights;
        // Filter stuff too close to entrance
        spawn_map.retain(|_, &mut w| w >= MIN_DISTANCE_FROM_ENTRANCE);
        // Don't need weights anymore, convert to Vec.
        let mut spawn_locs: Vec<Location> = spawn_map.into_iter().map(|(loc, _)| loc).collect();
        // Filter stuff next to walls, only spawn in open areas
        spawn_locs.retain(|&loc| {
            Dir6::iter()
                .map(|&d| (map.get(loc + d) == Floor) as u32)
                .sum::<u32>() > 3
        });

        let mut spawn_locs = rand::sample(rng, spawn_locs.iter(), 20);
        let n_spawns = spawn_locs.len();

        let items = Form::filter(|f| f.is_item() && f.at_depth(depth));
        for &loc in spawn_locs.drain(0..n_spawns / 2) {
            self.spawns.push((
                loc,
                form::rand(rng, &items)
                    .expect("No item spawn")
                    .loadout
                    .clone(),
            ))
        }

        let mobs = Form::filter(|f| f.is_mob() && f.at_depth(depth));
        for &loc in spawn_locs {
            self.spawns.push((
                loc,
                form::rand(rng, &mobs)
                    .expect("No mob spawn")
                    .loadout
                    .clone(),
            ))
        }


        // Map to actual terrains
        self.terrain.extend(map.iter().map(|(&loc, &t)| {
            let t = match t {
                Outside => Terrain::Empty,
                Unused | Border | Wall => Terrain::Rock,
                Floor => Terrain::Ground,
                Door => Terrain::Door,
            };
            (loc, t)
        }));

        // XXX: This thing needs to be more automatic
        // Make the backportal cell have transparent terrain


        return exit_loc;

        fn can_be_path_down_opening(
            map: &Field<Prototerrain>,
            origin: Location,
            loc: Location,
        ) -> bool {
            const MIN_EXIT_DISTANCE: i32 = 12;

            loc.metric_distance(origin) > MIN_EXIT_DISTANCE &&
                map.get(loc + Dir6::North) == Floor &&
                map.get(loc + Dir6::Northeast) != Floor &&
                map.get(loc + Dir6::Southeast) != Floor &&
                map.get(loc + Dir6::South) != Floor &&
                map.get(loc + Dir6::Southwest) != Floor &&
                map.get(loc + Dir6::Northwest) != Floor
        }
    }

    /// Make a cave entrance going down.
    fn cave_entrance(&mut self, loc: Location, cave_start: Location) {
        const DOWNBOUND_ENCLOSURE: [(i32, i32); 5] = [(1, 0), (0, 1), (2, 1), (1, 2), (2, 2)];

        for &v in &DOWNBOUND_ENCLOSURE {
            let loc = loc + vec2(v.0, v.1);
            self.terrain.insert(loc, Terrain::Rock);
        }

        self.terrain.insert(loc, Terrain::Ground);
        self.terrain.insert(loc + vec2(1, 1), Terrain::Ground);

        // Connect the points
        self.portal(loc + vec2(1, 1), cave_start);
        self.portal(cave_start - vec2(1, 1), loc);
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


#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Prototerrain {
    /// Area completely outside the map to be generated.
    Outside,
    /// Area that should be filled with map content but hasn't been yet.
    Unused,
    /// Area that the player cannot enter but that may be visible on screen.
    Border,
    Floor,
    Wall,
    Door,
}

impl Default for Prototerrain {
    fn default() -> Self { Prototerrain::Outside }
}

fn screen_map(origin: Location) -> Field<Prototerrain> {
    let mut ret = Field::new();

    // Cells two cells away from the live area at the bottom of the screen can still affect
    // the visible terrain via cell reshaping, so add those as well to the border.
    const BORDER_MASK: [(i32, i32); 9] = [
        (-1, -1),
        (0, -1),
        (1, 0),
        (1, 1),
        (0, 1),
        (-1, 0),
        (2, 1),
        (2, 2),
        (1, 2),
    ];

    for loc in origin.sector().iter() {
        // Paint border for every cell, all except the one from the cells at the actual border will
        // be overwritten.
        for d in &BORDER_MASK {
            let vec = vec2(d.0, d.1);
            if ret.get(loc + vec) == Default::default() {
                ret.set(loc + vec, Prototerrain::Border);
            }
        }

        ret.set(loc, Prototerrain::Unused);
    }

    ret
}

fn entry_cave_enclosure(map: &mut Field<Prototerrain>, entrance: Location) {
    use self::Prototerrain::*;

    debug_assert_eq!(map.get(entrance), Unused);

    const UPBOUND_ENCLOSURE: [(i32, i32); 7] = [
        (-2, -2),
        (-1, -2),
        (0, -1),
        (-1, 0),
        (-2, -1),
        (1, 0),
        (0, 1),
    ];

    for &v in &UPBOUND_ENCLOSURE {
        let loc = entrance + vec2(v.0, v.1);
        map.set(loc, Wall);
    }

    // Portal site
    map.set(entrance - vec2(1, 1), Floor);
    // Entrance cell
    map.set(entrance, Floor);
    // Enclosure mouth
    map.set(entrance + vec2(1, 1), Floor);
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
                sector_origin.v2_at(loc).expect("Sector points are not Euclidean");

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
}

impl<'a> mapgen::Dungeon for SectorDigger<'a> {
    type Prefab = Room;

    /// Return a random prefab room.
    fn sample_prefab<R: Rng>(&mut self, rng: &mut R) -> Self::Prefab { rng.gen() }

    /// Add a large open continuous region to dungeon.
    fn dig_chamber<I: IntoIterator<Item=mapgen::Point2D>>(&mut self, area: I) {
        for pos in area.into_iter() {
            let loc = self.loc(pos);
            self.spawn_region.insert(loc);
            self.set(loc, Terrain::Ground);
        }
    }

    /// Add a narrow corridor to dungeon.
    fn dig_corridor<I: IntoIterator<Item=mapgen::Point2D>>(&mut self, path: I) {
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
