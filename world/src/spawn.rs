use rand::Rng;
use rand::distributions::{Weighted, WeightedChoice, IndependentSample};
use components::{Category};
use entity::Entity;
use action;
use Biome;
use location::{Location};
use world;
use ecs::{ComponentAccess};

/// Representation for an abstract spawnable object. Does not refer to
/// concrete entity prototypes and can be used before the prototype set has
/// been initialized.
#[derive(Copy, Clone, Eq, PartialEq, Debug, RustcEncodable, RustcDecodable)]
pub struct Spawn {
    // XXX: this is almost the exact same thing as the component Spawn...
    // Though this one can become more complex if we want greater detail, the
    // component thing is just for sampling, this could be used to eg. create
    // specific entities by name.
    category_mask: u32,
    biome_mask: u32,
    depth: i32,
}

impl Spawn {
    /// Empty categories or biomes are treated as matching any category or
    /// biome.
    pub fn new(depth: i32, categories: Vec<Category>, biomes: Vec<Biome>) -> Spawn {
        let category_mask = if categories.is_empty() { -1 }
        else { categories.into_iter().fold(0, |a, x| a | x as u32) };

        let biome_mask = if biomes.is_empty() { -1 }
        else { biomes.into_iter().fold(0, |a, x| a | x as u32) };

        Spawn {
            category_mask: category_mask,
            biome_mask: biome_mask,
            depth: depth,
        }
    }

    pub fn spawn<R: Rng>(&self, rng: &mut R, loc: Location) -> Entity {
        // XXX: Optimization option: memoize WeightedChoices for the biomask,
        // catmask, depth tuples.
        let mut items: Vec<Weighted<Entity>> = action::entities()
            .filter_map(|e| world::with(|w| {
                if let Some(spawn) = w.spawns().get_local(e) {
                    if spawn.min_depth <= self.depth
                        && self.biome_mask & (spawn.biome as u32) != 0
                        && self.category_mask & (spawn.category as u32) != 0 {
                        return Some(Weighted { weight: spawn.commonness as u32, item: e });
                    }
                }
                return None;
            }))
            .collect();
        assert!(!items.is_empty(), format!("Couldn't find spawns for {:?}", self));
        let dist = WeightedChoice::new(items.as_mut_slice());
        let e = dist.ind_sample(rng);
        e.clone_at(loc);
        e
    }
}
