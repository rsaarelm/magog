use std::rand::StdRng;
use std::rand::Rng;
use std::rand::SeedableRng;
use serialize::{Decodable, Decoder, Encodable, Encoder};
use std::collections::HashMap;
use terrain::TerrainType;
use location::Location;
use mapgen;
use egg::Egg;
use mob;
use mob::{MobSpec};
use {AreaSpec};
use components::{Kind};

// Note to maintainer: Due to the way serialization works, Area *must* be
// generated to have exactly the same contents every time given the same seed
// value. That means absolutely no outside sources of randomness. Only use an
// internal generator initialized with the given seed value. And watch out for
// surprise randomness. A map generator in a previous project used memory
// addresses of temporary structures as indexing keys, and ended up with a
// nondeterminism bug that depended on the numerical order of the arbitrary
// address values.

#[deriving(Decodable, Encodable)]
struct AreaSeed {
    pub rng_seed: u32,
    pub spec: AreaSpec,
}

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
    eggs: Vec<(Egg, Location)>,
}

impl<E, D:Decoder<E>> Decodable<D, E> for Area {
    fn decode(d: &mut D) -> Result<Area, E> {
        let seed: AreaSeed = try!(Decodable::decode(d));
        Ok(Area::new(seed.rng_seed, seed.spec))
    }
}

impl<E, S:Encoder<E>> Encodable<S, E> for Area {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        self.seed.encode(s)
    }
}

impl Area {
    pub fn new(rng_seed: u32, spec: AreaSpec) -> Area {
        let num_eggs = 32i;

        let mut terrain = HashMap::new();
        let mut rng: StdRng = SeedableRng::from_seed([rng_seed as uint + spec.depth as uint].as_slice());
        mapgen::gen_herringbone(
            &mut rng,
            &spec,
            |p, t| {terrain.insert(Location::new(0, 0) + p, t);});

        // Generate open slots that can be used to spawn stuff.
        let mut opens = Vec::new();
        for (&loc, t) in terrain.iter() {
            // No connectivity analysis yet, trusting that herringbone map has
            // total connectivity. Later on, use Dijkstra map that spreads
            // from entrance/exit as a reachability floodfill to do something
            // cleverer here.
            if !t.blocks_walk() {
                opens.push(loc);
            }
        }
        rng.shuffle(opens.as_mut_slice());

        let entrance = opens.swap_remove(0).unwrap();

        let viable_mobs: Vec<MobSpec> = mob::MOB_SPECS.iter()
            .filter(|m| m.area_spec.can_hatch_in(&spec))
            .map(|&x| x).collect();

        let mut eggs = vec![];

        assert!(viable_mobs.len() > 0, "Area has no viable mob spawns");
        for _ in range(0, num_eggs) {
            if let Some(loc) = opens.swap_remove(0) {
                let mob = rng.choose(viable_mobs.as_slice()).unwrap();
                eggs.push((Egg::new(Kind::Mob(mob.typ)), loc));
            } else {
                // Ran out of open space.
                break;
            }
        }

        Area {
            seed: AreaSeed { rng_seed: rng_seed, spec: spec },
            terrain: terrain,
            _open_slots: opens,
            player_entrance: entrance,
            eggs: eggs,
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
    pub fn get_eggs(&self) -> Vec<(Egg, Location)> { self.eggs.clone() }

    pub fn player_entrance(&self) -> Location {
        self.player_entrance
    }

    fn default_terrain(&self, _loc: Location) -> TerrainType {
        self.seed.spec.biome.default_terrain()
    }
}
