use calx::{self, RngExt, WeightedChoice};
use location::{Location, Sector};
use map::Map;
use rand::seq;
use rand::Rng as _Rng;
use spec::{self, EntitySpawn};
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;
use vaults;
use {Distribution, Rng};

/// Descriptor for different regions of the game world for spawn distributions.
pub struct Biome {
    depth: i32,
    // TODO: Branch specifications go here.
}

impl Biome {
    pub fn new(depth: i32) -> Biome { Biome { depth } }
}

struct Entrance(Arc<Map>);

impl Distribution<Entrance> for Biome {
    fn sample(&self, rng: &mut Rng) -> Entrance {
        Entrance(rng.pick_slice(&vaults::ENTRANCES).unwrap().clone())
    }
}

struct Room(Arc<Map>);

impl Distribution<Room> for Biome {
    fn sample(&self, rng: &mut Rng) -> Room {
        if rng.one_chance_in(12) {
            // Make a vault sometimes.
            Room(rng.pick_slice(&vaults::VAULTS).unwrap().clone())
        } else {
            // Make a procgen room normally.
            let mut map = Map::new_plain_room(rng);
            let floor_area = map.open_ground();
            let num_spawns = rng.gen_range(0, floor_area.len() / 8 + 1);

            for pos in seq::sample_slice(rng, &floor_area, num_spawns) {
                map.push_spawn(pos, self.sample(rng));
            }

            Room(Arc::new(map))
        }
    }
}

struct Exit(Arc<Map>);

impl Distribution<Exit> for Biome {
    fn sample(&self, rng: &mut Rng) -> Exit {
        Exit(rng.pick_slice(&vaults::EXITS).unwrap().clone())
    }
}

/// Biome-sampleable newtype for dungeon level maps.
pub struct Dungeon(pub Map);

impl Distribution<Dungeon> for Biome {
    // TODO: When there are multiple types of map generator (eg. Caves, RoomsAndCorridors),
    // write each their own newtype and have this master generator choose between them based on
    // Biome.
    fn sample(&self, rng: &mut Rng) -> Dungeon {
        fn gen(rng: &mut Rng, biome: &Biome) -> Result<Map, Box<Error>> {
            debug!("Starting mapgen");
            let mut gen = Map::new_base(Sector::points().filter(|p| {
                !Location::new(p.x as i16, p.y as i16, 0).is_next_to_diagonal_sector()
            }));

            // TODO: Helper function room picker
            let room: Entrance = biome.sample(rng);
            debug!("Placing entrance");
            gen.place_room(rng, &*room.0)?;

            loop {
                let room: Room = biome.sample(rng);
                debug!("Adding room");
                if gen.place_room(rng, &*room.0).is_err() {
                    break;
                }
            }

            /*
            let room: Map = self.sample(rng);
            let sites = gen.room_positions(&room);
            let pos = *seq::sample_iter(rng, &sites, 1).unwrap()[0];
            gen.place_room_at(pos, &room);
            */

            debug!("Placing exit");
            let room = rng.pick_slice(&vaults::EXITS).unwrap();
            gen.place_room(rng, &*room)?;

            if let Some(map) = gen.join_disjoint_regions(rng) {
                Ok(map)
            } else {
                die!("Failed to join map");
            }
        }

        Dungeon(calx::retry_gen(16, rng, |rng| gen(rng, self)).expect("Couldn't generate map"))
    }
}

impl Distribution<EntitySpawn> for Biome {
    fn sample(&self, rng: &mut Rng) -> EntitySpawn {
        let item = spec::iter_specs()
            .weighted_choice(rng, |item| {
                if item.rarity() == 0.0 || item.min_depth() > self.depth {
                    0.0
                } else {
                    1.0 / item.rarity()
                }
            })
            .unwrap();

        EntitySpawn::from_str(item.name()).unwrap()
    }
}
