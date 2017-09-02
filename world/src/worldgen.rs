use Prefab;
use Rng;
use calx_grid;
use calx_grid::Dir6;
use euclid::{vec2, Vector2D};
use field::Field;
use form::Form;
use location::{Location, Portal};
use mapfile;
use rand::{self, Rand, SeedableRng};
use serde;
use std::cmp;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::io::Cursor;
use std::slice;
use terrain::Terrain;
use world::Loadout;

/// Static generated world.
pub struct Worldgen {
    seed: u32,
    terrain: Field<Terrain>,
    portals: HashMap<Location, Portal>,
    spawns: Vec<(Location, Loadout)>,
    player_entry: Location,
}

impl Worldgen {
    pub fn new(seed: u32) -> Worldgen {
        let mut ret = Worldgen {
            seed: seed,
            terrain: Field::new(),
            portals: HashMap::new(),
            spawns: Vec::new(),
            player_entry: Location::new(0, 0, 0),
        };

        let mut rng: Rng = SeedableRng::from_seed([seed, seed, seed, seed]);
        //ret.sprint_map();
        ret.gen_caves(&mut rng, Location::new(0, 0, 0));

        ret
    }

    fn sprint_map(&mut self) {
        const SPRINT_PREFAB: &'static str = include_str!("../../sprint.ron");

        let sprint_prefab = mapfile::load_prefab(&mut Cursor::new(SPRINT_PREFAB))
            .expect("Corrupt sprint file");

        // Top-left of the default on-screen area.
        let origin = Location::new(-21, -22, 0);
        self.load_prefab(origin, &sprint_prefab);
    }

    fn load_prefab(&mut self, origin: Location, prefab: &Prefab) {
        for (p, &(ref terrain, ref entities)) in prefab.iter() {
            let loc = origin + p;

            self.terrain.set(loc, *terrain);

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

    pub fn get_terrain(&self, loc: Location) -> Terrain { self.terrain.get(loc) }

    pub fn get_portal(&self, loc: Location) -> Option<Location> {
        self.portals.get(&loc).map(|&p| loc + p)
    }

    pub fn spawns(&self) -> slice::Iter<(Location, Loadout)> { self.spawns.iter() }

    pub fn player_entry(&self) -> Location { self.player_entry }

    fn gen_caves<R: rand::Rng>(&mut self, rng: &mut R, entrance: Location) {
        use self::Prototerrain::*;

        let mut cells_to_dig = 700;

        let mut map = screen_map(Location::new(0, 0, entrance.z));

        debug_assert!(map.get(entrance) == Unused);

        // Create portal enclosure
        // TODO: Backportal
        entry_cave_enclosure(&mut map, entrance);

        let entrance = entrance + vec2(1, 1);

        let mut edge: BTreeSet<Location> = Dir6::iter()
            .map(|&d| entrance + vec2(2, 2) + d)
            .filter(|&loc| map.get(loc) == Unused)
            .collect();

        // Arbitrary long iteration, should break after digging a sufficient number of cells before
        // this.
        for _ in 0..10000 {
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
        for (loc, c) in cells.into_iter() {
            // Clear pillars
            if c == Unused {
                let adjacent_floors = Dir6::iter().filter(|d| map.get(loc + **d) == Floor).count();

                if adjacent_floors == 6 {
                    map.set(loc, Floor);
                }
            }
        }

        // Map to actual terrains
        self.terrain.extend(map.iter().map(|(&loc, &t)| {
            let t = match t {
                Outside => Terrain::Empty,
                Unused => Terrain::Rock,
                Border => Terrain::Rock,
                Floor => Terrain::Ground,
                Wall => Terrain::Rock,
                Door => Terrain::Door,
            };
            (loc, t)
        }));
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

    for &v in ::onscreen_locations() {
        let loc = origin + v;
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

    debug_assert!(map.get(entrance) == Unused);

    const DOWNBOUND_ENCLOSURE: [(i32, i32); 7] = [
        (-2, -2),
        (-1, -2),
        (0, -1),
        (-1, 0),
        (-2, -1),
        (1, 0),
        (0, 1),
    ];

    for &v in &DOWNBOUND_ENCLOSURE {
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
