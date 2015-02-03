use std::rand::StdRng;
use std::rand::Rng;
use std::rand::SeedableRng;
use std::iter;
use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};
use std::collections::HashMap;
use terrain::TerrainType;
use location::Location;
use mapgen;
use {AreaSpec};
use components::{Category};
use spawn::Spawn;

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
    /// Valid slots to spawn things in, basically open floor and connected to
    /// areas the player can reach. (Does not handle stuff like monsters that
    /// spawn in water for now.)
    _open_slots: Vec<Location>,
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
        let num_mobs = 32;
        let num_items = 24;

        let mut terrain = HashMap::new();
        let mut rng: StdRng = SeedableRng::from_seed([rng_seed as usize + spec.depth as usize].as_slice());
        mapgen::gen_herringbone(
            &mut rng,
            &spec,
            |x, y, z, t| {terrain.insert(Location::new(x as i8, y as i8, z as i8), t);});

        // Generate open slots that can be used to spawn stuff.
        let mut opens = Vec::new();
        for (&loc, t) in terrain.iter() {
            // No connectivity analysis yet, trusting that herringbone map has
            // total connectivity. Later on, use Dijkstra map that spreads
            // from entrance/exit as a reachability floodfill to do something
            // cleverer here.
            if loc.z == 0 && !t.blocks_walk() &&
                terrain.get(&loc.below()).map_or(false, |t| t.is_solid()) {
                opens.push(loc);
            }
        }
        rng.shuffle(opens.as_mut_slice());

        let entrance = opens.pop().unwrap();

        let mut spawns = vec![];

        // XXX: copy-pasting the space-finding code.
        spawns.extend(
            iter::repeat(Spawn::new(spec.depth, vec![Category::Mob], vec![spec.biome]))
            .take(num_mobs)
            .filter_map(|spawn|
            if let Some(loc) = opens.pop() { Some((spawn, loc)) } else { None }));

        spawns.extend(
            iter::repeat(Spawn::new(spec.depth, vec![Category::Item], vec![spec.biome]))
            .take(num_items)
            .filter_map(|spawn|
            if let Some(loc) = opens.pop() { Some((spawn, loc)) } else { None }));

        Area {
            seed: AreaSeed { rng_seed: rng_seed, spec: spec },
            terrain: terrain,
            _open_slots: opens,
            player_entrance: entrance,
            spawns: spawns,
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
