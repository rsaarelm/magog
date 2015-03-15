use rand::StdRng;
use rand::Rng;
use rand::SeedableRng;
use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};
use std::collections::HashMap;
use terrain::TerrainType;
use location::Location;
use mapgen;
use {AreaSpec, Biome};
use components::{Category};
use spawn::Spawn;
use dir6::Dir6;

// Note to maintainer: Due to the way serialization works, Area *must* be
// generated to have exactly the same contents every time given the same seed
// value. That means absolutely no outside sources of randomness. Only use an
// internal generator initialized with the given seed value. And watch out for
// surprise randomness. A map generator in a previous project used memory
// addresses of temporary structures as indexing keys, and ended up with a
// nondeterminism bug that depended on the numerical order of the arbitrary
// address values.

#[derive(Copy, Clone, RustcDecodable, RustcEncodable)]
struct AreaSeed {
    pub rng_seed: u32,
    pub spec: AreaSpec,
}

#[derive(Clone)]
/// Immutable procedurally generated terrain initialized on random seed.
pub struct Area {
    /// Random number generator seed. Must uniquely define the Area contents.
    pub seed: AreaSeed,
    /// Stored terrain.
    pub terrain: HashMap<Location, TerrainType>,
    /// Where the player should enter the area.
    player_entrance: Location,
    /// Non-player entities to create when first initializing the map.
    spawns: Vec<(Spawn, Location)>,
    pub biomes: HashMap<Location, Biome>,
}

impl Decodable for Area {
    fn decode<D: Decoder>(d: &mut D) -> Result<Area, D::Error> {
        let seed: AreaSeed = try!(Decodable::decode(d));
        Ok(Area::new(seed.rng_seed, seed.spec))
    }
}

impl Encodable for Area {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        self.seed.encode(s)
    }
}

impl Area {
    pub fn new(rng_seed: u32, spec: AreaSpec) -> Area {
        let mut terrain = HashMap::new();
        let mut biomes = HashMap::new();
        let mut rng: StdRng = SeedableRng::from_seed(&[rng_seed as usize + spec.depth as usize][..]);
        mapgen::gen_herringbone(
            &mut rng,
            &spec,
            |p, t| {terrain.insert(Location::new(0, 0) + p, t);},
            |p, b| {biomes.insert(Location::new(0, 0) + p, b);});

        // Generate open slots that can be used to spawn stuff.

        // No connectivity analysis yet, trusting that herringbone map has
        // total connectivity. Later on, use Dijkstra map that spreads from
        // entrance/exit as a reachability floodfill to do something cleverer
        // here.
        let mut outdoors: Vec<Location> = terrain.iter()
            .filter(|&(loc, &t)| t.valid_spawn_spot() && biomes.get(loc) == Some(&Biome::Overland))
            .map(|(&loc, _)| loc)
            .collect();
        rng.shuffle(outdoors.as_mut_slice());

        let mut bases: Vec<Location> = terrain.iter()
            .filter(|&(loc, &t)| t.valid_spawn_spot() && biomes.get(loc) == Some(&Biome::Base))
            .map(|(&loc, _)| loc)
            .collect();
        rng.shuffle(bases.as_mut_slice());

        let entrance = outdoors.pop().unwrap();

        // Phage entrance crater
        terrain.insert(entrance, TerrainType::Pod);
        terrain.insert(entrance + Dir6::from_int(0).to_v2(), TerrainType::CraterN);
        terrain.insert(entrance + Dir6::from_int(1).to_v2(), TerrainType::CraterNE);
        terrain.insert(entrance + Dir6::from_int(2).to_v2(), TerrainType::CraterSE);
        terrain.insert(entrance + Dir6::from_int(3).to_v2(), TerrainType::CraterS);
        terrain.insert(entrance + Dir6::from_int(4).to_v2(), TerrainType::CraterSW);
        terrain.insert(entrance + Dir6::from_int(5).to_v2(), TerrainType::CraterNW);

        let mut spawns = vec![];

        for _ in 0..(rng.gen_range(40, 60)) {
            let loc = outdoors.pop().unwrap();
            spawns.push((Spawn::new(spec.depth, vec![Category::Mob], vec![Biome::Overland]), loc));
        }

        for _ in 0..(rng.gen_range(30, 50)) {
            let loc = bases.pop().unwrap();
            spawns.push((Spawn::new(spec.depth, vec![Category::Mob], vec![Biome::Base]), loc));
        }

        Area {
            seed: AreaSeed { rng_seed: rng_seed, spec: spec },
            terrain: terrain,
            player_entrance: entrance,
            spawns: spawns,
            biomes: biomes,
        }
    }

    /// Return terrain at given location.
    pub fn terrain(&self, loc: Location) -> TerrainType {
        match self.terrain.get(&loc) {
            Some(&t) => t,
            None => self.default_terrain(loc)
        }
    }

    /// List the objects that should be hatched in the world during init. This
    /// is tied to map generation, so it goes in the area module.
    pub fn get_spawns(&self) -> Vec<(Spawn, Location)> { self.spawns.clone() }

    pub fn player_entrance(&self) -> Location {
        self.player_entrance
    }

    fn default_terrain(&self, _loc: Location) -> TerrainType {
        self.seed.spec.biome.default_terrain()
    }
}
