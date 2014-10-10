use std::rand::StdRng;
use std::rand::SeedableRng;
use serialize::{Decodable, Decoder, Encodable, Encoder};
use std::collections::hashmap::HashMap;
use terrain::TerrainType;
use terrain;
use location::Location;
use mapgen;
use world;

/// Immutable procedurally generated terrain initialized on random seed.
pub struct Area {
    seed: u32,
    terrain: HashMap<Location, TerrainType>,
}

impl<E, D:Decoder<E>> Decodable<D, E> for Area {
    fn decode(d: &mut D) -> Result<Area, E> {
        Ok(Area::new(try!(d.read_u32())))
    }
}

impl<E, S:Encoder<E>> Encodable<S, E> for Area {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        s.emit_u32(self.seed)
    }
}

impl Area {
    pub fn new(seed: u32) -> Area {
        let mut terrain = HashMap::new();
        let mut rng: StdRng = SeedableRng::from_seed([seed as uint].as_slice());
        // TODO: Underground areas.
        mapgen::gen_herringbone(
            &mut rng,
            &mapgen::AreaSpec::new(mapgen::Overland, 1),
            |p, t| {terrain.insert(Location::new(0, 0) + p, t);});

        Area {
            seed: seed,
            terrain: terrain,
        }
    }

    /// Return terrain at given location.
    pub fn terrain(&self, loc: Location) -> TerrainType {
        match self.terrain.find(&loc) {
            Some(&t) => t,
            None => self.default_terrain(loc)
        }
    }

    /// Fill the world with the objects that come with the procedural world.
    /// This must only be called exactly once when initializing the world.
    pub fn populate(&self) {
        let w = world::get();
        assert!(!w.borrow().flags.populated, "Calling populate more than once");

        println!("TODO area::populate");

        w.borrow_mut().flags.populated = true;
    }

    fn default_terrain(&self, _loc: Location) -> TerrainType {
        // TODO: Different default terrains in different biomes.
        terrain::Rock
    }
}
