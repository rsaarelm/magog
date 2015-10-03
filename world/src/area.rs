use rand::StdRng;
use rand::SeedableRng;
use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};
use std::collections::BTreeMap;
use calx_ecs::Entity;
use calx::{Field, Backdrop, ConstBackdrop, Patch};
use location::Location;
use content::{self, AreaSpec, TerrainType, Biome};
use rand::Rng;
use spawn::Spawn;
use world::World;
use query;
use action;

pub type TerrainField = Field<Location, TerrainType, ConstBackdrop<TerrainType>, BTreeMap<Location, TerrainType>>;

pub fn start_level(w: &mut World, depth: i32) {
    clear_nonplayers(w);
    init_area(w, depth);

    /*
    // TODO: Get spawns working again.
    let mut rng: StdRng = SeedableRng::from_seed(&[seed as usize + depth as usize][..]);
    for (spawn, loc) in world::with(|w| w.area.get_spawns()).into_iter() {
        spawn.spawn(&mut rng, loc);
    }
    */

    let start_loc = w.area.player_entrance();
    // Either reuse the existing player or create a new one.
    match query::player(w) {
        Some(player) => {
            action::forget_map(w, player);
            action::place_entity(w, player, start_loc);
        }
        None => {
            // TODO: Use factory.
            use calx::color;
            use content::Brush;
            use components::{Desc, MapMemory, Health, Brain, BrainState, Alignment};
            use stats::{Stats};
            use stats::Intrinsic::*;
            let player = w.ecs.make();
            w.ecs.desc.insert(player, Desc::new("player", Brush::Human, color::AZURE));
            w.ecs.map_memory.insert(player, MapMemory::new());
            w.ecs.brain.insert(player, Brain { state: BrainState::PlayerControl, alignment: Alignment::Good });
            w.ecs.stats.insert(player, Stats::new(10, &[Hands]).mana(5));
            action::recompose_stats(w, player);

            w.flags.player = Some(player);

            action::place_entity(w, player, start_loc);
            // TODO: run FOV
        }
    };
    w.flags.camera = start_loc;
}

fn clear_nonplayers(w: &mut World) {
    let po = query::player(w);
    let entities: Vec<Entity> = w.ecs.iter().map(|&e| e).collect();
    for e in entities.into_iter() {
        // Don't destroy player or player's inventory.
        if let Some(p) = po {
            if e == p || w.spatial.contains(p, e) {
                continue;
            }
        }

        if query::location(w, e).is_some() {
            w.ecs.remove(e);
        }
    }
}

pub fn next_level(w: &mut World) {
    // This is assuming a really simple, original Rogue style descent-only, no
    // persistent maps style world.
    let new_depth = query::current_depth(w) + 1;
    start_level(w, new_depth);
    // 1st level is the overworld, so we want to call depth=2, first dungeon
    // level as "Depth 1" in game.
    caption!("Depth {}", new_depth - 1);
}

fn init_area(w: &mut World, depth: i32) {
    let biome = match depth {
        1 => Biome::Overland,
        _ => Biome::Dungeon,
    };

    let spec = AreaSpec::new(biome, depth);
    let mut rng: StdRng = SeedableRng::from_seed(&[w.flags.seed as usize + spec.depth as usize][..]);
    let static_area = if spec.depth == 1 {
        content::herringbone(&mut rng, &spec)
    } else {
        content::rooms_and_corridors(&mut rng, spec.depth)
    };

    let mut terrain = TerrainField::new(ConstBackdrop(
            match biome {
                Biome::Overland => TerrainType::Tree,
                _ => TerrainType::Rock
            }));

    let origin = Location::new(0, 0);
    for (&pos, &t) in static_area.terrain.iter() {
        terrain.set(origin + pos, t);
    }
}

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
            content::herringbone(&mut rng, &spec)
        } else {
            content::rooms_and_corridors(&mut rng, spec.depth)
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
