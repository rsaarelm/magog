use Prefab;
use field::Field;
use form::Form;
use location::{Location, Portal};
use mapfile;
use serde;
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
            terrain: Field::new(Terrain::Empty),
            portals: HashMap::new(),
            spawns: Vec::new(),
            player_entry: Location::new(0, 0, 0),
        };

        ret.sprint_map();

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
