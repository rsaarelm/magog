//! Game world generation

use biome::{Biome, Dungeon};
use calx::{seeded_rng, RngExt};
use euclid::vec2;
use location::{Location, Portal, Sector};
use map::{Map, MapCell};
use serde;
use std::collections::HashMap;
use std::slice;
use terrain::Terrain;
use world::Loadout;
use Distribution;

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
            seed,
            terrain: HashMap::new(),
            portals: HashMap::new(),
            spawns: Vec::new(),
            player_entry: Location::new(0, 0, 0),
        };

        let mut rng: ::Rng = seeded_rng(&seed);

        const NUM_FLOORS: i32 = 10;

        let floors: Vec<Map> = (0..NUM_FLOORS)
            .map(|i| {
                let map: Dungeon = Biome::new(i + 1).sample(&mut rng);
                map.0
            })
            .collect();

        for i in 0..floors.len() {
            let depth = (i + 1) as i16;
            let origin = Sector::new(0, 0, depth as i16).origin();
            let map = &floors[i];

            if depth == 1 {
                ret.player_entry = origin + map.entrances()[0];
            }

            for (
                vec,
                MapCell {
                    terrain, spawns, ..
                },
            ) in map
            {
                let loc = origin + *vec;
                if *terrain != Terrain::Empty {
                    ret.terrain.insert(loc, *terrain);
                }

                for s in spawns {
                    ret.spawns.push((loc, s.sample(&mut rng)))
                }
            }

            // Connect to downstairs.
            if i < floors.len() - 1 {
                let mut other_origin = origin;
                other_origin.z += 1;
                let other_stairs = floors[i + 1].entrances();

                for &stair in &map.exits() {
                    let other = rng.pick_slice(&other_stairs).unwrap();
                    ret.make_stairs(origin + stair, other_origin + other);
                }
            }
        }

        ret
    }

    pub fn seed(&self) -> u32 { self.seed }

    pub fn get_terrain(&self, loc: Location) -> Terrain {
        if let Some(&t) = self.terrain.get(&loc) {
            t
        } else {
            self.default_terrain(loc)
        }
    }

    fn default_terrain(&self, _loc: Location) -> Terrain { Terrain::Rock }

    pub fn get_portal(&self, loc: Location) -> Option<Location> {
        self.portals.get(&loc).map(|&p| loc + p)
    }

    pub fn spawns(&self) -> slice::Iter<(Location, Loadout)> { self.spawns.iter() }

    pub fn player_entry(&self) -> Location { self.player_entry }

    /// Punch a (one-way) portal between two points.
    fn portal(&mut self, origin: Location, destination: Location) {
        self.portals
            .insert(origin, Portal::new(origin, destination));
    }

    /// Make a two-way stairwell portal.
    fn make_stairs(&mut self, downstairs: Location, upstairs: Location) {
        self.portal(upstairs, downstairs - vec2(1, 1));
        self.portal(downstairs, upstairs + vec2(1, 1));
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

#[cfg(test)]
mod test {
    use super::*;

    /*
    // FIXME: Worldgen takes too long right now.
    #[test]
    fn test_determinism() {
        use rand::{self, Rng};

        let mut rng = rand::thread_rng();

        let seed: u32 = rng.gen();
        println!("Testing worldgen determinism with seed {}", seed);
        let mut gen = Worldgen::new(seed);

        // Build the value repeatedly using the same seed and see that they are all equal.
        for _ in 1..4 {
            let second = Worldgen::new(seed);

            assert_eq!(gen.seed, second.seed);
            // These can make huge printouts so don't use assert_eq that would try to print them to
            // stdout
            assert!(gen.terrain == second.terrain);
            assert!(gen.portals == second.portals);
            assert_eq!(gen.player_entry, second.player_entry);

            gen = second;
        }
    }
    */
}
