use rand::Rng;
use rand::distributions::{Weighted, WeightedChoice, IndependentSample};
use calx_ecs::Entity;
use content::{FormType};
use action;
use content::{Biome};
use location::{Location};
use world;

/// Representation for an abstract spawnable object. Does not refer to
/// concrete entity prototypes and can be used before the prototype set has
/// been initialized.
#[derive(Copy, Clone, Eq, PartialEq, Debug, RustcEncodable, RustcDecodable)]
pub struct Spawn {
    // XXX: this is almost the exact same thing as the component Spawn...
    // Though this one can become more complex if we want greater detail, the
    // component thing is just for sampling, this could be used to eg. create
    // specific entities by name.
    spawn_type: FormType,
    biome_mask: u32,
    depth: i32,
}

impl Spawn {
    /// Empty categories or biomes are treated as matching any category or
    /// biome.
    pub fn new(depth: i32, spawn_type: FormType, biomes: Vec<Biome>) -> Spawn {
        let biome_mask = if biomes.is_empty() { ::std::u32::MAX }
        else { biomes.into_iter().fold(0, |a, x| a | x as u32) };

        Spawn {
            spawn_type: spawn_type,
            biome_mask: biome_mask,
            depth: depth,
        }
    }

    pub fn spawn<R: Rng>(&self, rng: &mut R, loc: Location) -> Entity {
        unimplemented!();
        /*
        // XXX: Optimization option: memoize WeightedChoices for the biomask,
        // catmask, depth tuples.
        let mut items: Vec<Weighted<Entity>> = action::entities()
            .filter_map(|e| world::with(|w| {
                if let Some(spawn) = w.spawns().get_local(e) {
                    if spawn.min_depth <= self.depth
                        && self.biome_mask & (spawn.biome as u32) != 0
                        && spawn.category.is_a(self.spawn_type) {
                        return Some(Weighted { weight: spawn.commonness, item: e });
                    }
                }
                return None;
            }))
            .collect();
        assert!(!items.is_empty(), format!("Couldn't find spawns for {:?}", self));
        let dist = WeightedChoice::new(&mut items);
        let e = dist.ind_sample(rng);
        e.clone_at(loc);
        e
        */
    }
}
