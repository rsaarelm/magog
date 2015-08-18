use rand::StdRng;
use rand::SeedableRng;
use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};
use std::collections::BTreeMap;
use location::Location;
use content::{herringbone, rooms_and_corridors, AreaSpec, TerrainType};
use spawn::Spawn;

// Note to maintainer: Due to the way serialization works, Area *must* be
// generated to have exactly the same contents every time given the same seed
// value. That means absolutely no outside sources of randomness. Only use an
// internal generator initialized with the given seed value. And watch out for
// surprise randomness. A map generator in a previous project used memory
// addresses of temporary structures as indexing keys, and ended up with a
// nondeterminism bug that depended on the numerical order of the arbitrary
// address values. Relying on the iteration order of collections::HashMap has
// introduced at least one nondeterminism bug to map generation.

#[derive(Copy, Clone, RustcDecodable, RustcEncodable)]
pub struct AreaSeed {
    pub rng_seed: u32,
    pub spec: AreaSpec,
}

#[derive(Clone)]
/// Immutable procedurally generated terrain initialized on random seed.
pub struct Area {
    /// Random number generator seed. Must uniquely define the Area contents.
    pub seed: AreaSeed,
    /// Stored terrain.
    pub terrain: BTreeMap<Location, TerrainType>,
    /// Where the player should enter the area.
    player_entrance: Location,
    /// Non-player entities to create when first initializing the map.
    spawns: Vec<(Spawn, Location)>,
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
        let mut terrain = BTreeMap::new();
        let origin = Location::new(0, 0);
        let mut rng: StdRng = SeedableRng::from_seed(&[rng_seed as usize + spec.depth as usize][..]);
        let static_area = if spec.depth == 1 {
            herringbone(&mut rng, &spec)
        } else {
            rooms_and_corridors(&mut rng, spec.depth)
        }.map_spawns(|s| Spawn::new(spec.depth, s, vec![spec.biome]));

        for (&p, &t) in static_area.terrain.iter() {
            terrain.insert(origin + p, t);
        }

        Area {
            seed: AreaSeed { rng_seed: rng_seed, spec: spec },
            terrain: terrain,
            player_entrance: origin + static_area.player_entrance,
            spawns: static_area.spawns.iter().map(|&(p, s)| (s, origin + p)).collect(),
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
