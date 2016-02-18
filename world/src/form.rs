use std::collections::HashMap;
use std::rc::Rc;
use rand::Rng;
use rand::distributions::{Weighted, WeightedChoice, IndependentSample};
use calx_ecs::Entity;
use calx_color::color::*;
use calx_color::Rgba;
use content::{Biome, FormType, Brush, AreaSpec};
use content::Biome::*;
use stats::{Stats, Intrinsic};
use stats::Intrinsic::*;
use components::{Desc, Brain, Health};
use world::{World, Loadout};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Forms {
    Player,
    Dreg,
    Snake,
    Ooze,
}

/// Forms are the prototypes for the entities you create.
#[derive(Debug)]
pub struct Form {
    /// Types of areas where this entity can spawn.
    pub biome: Biome,
    /// Weight of this entity in the random sampling distribution.
    ///
    /// The convention is that commonness 1000 is the average value. Things
    /// that never spawn randomly have commonness 0.
    pub commonness: u32,
    /// Minimum depth where the entity will show up. More powerful entities
    /// only start showing up in large depths.
    pub min_depth: i32,
    /// Type of thing.
    pub category: FormType,
    /// Actual components to set up the thing.
    pub loadout: Loadout,
}

impl Form {
    /// Create a standard form for a living creature.
    pub fn mob(name: &str,
               icon: Brush,
               color: Rgba,
               power: i32,
               intrinsics: &[Intrinsic])
               -> Form {
        Form {
            biome: Anywhere,
            commonness: 1000,
            min_depth: 0,
            category: FormType::Creature,
            loadout: Loadout::new()
                         .c(Stats::new(power, intrinsics))
                         .c(Desc::new(name, icon, color))
                         .c(Brain::enemy())
                         .c(Health::new()),
        }
    }

    pub fn common(mut self, commonness: u32) -> Form {
        self.commonness = commonness;
        self
    }

    pub fn depth(mut self, min_depth: i32) -> Form {
        self.min_depth = min_depth;
        self
    }

    pub fn biome(mut self, biome: Biome) -> Form {
        self.biome = biome;
        self
    }

    /// Build a new entity with this form.
    pub fn build(&self, w: &mut World) -> Entity {
        self.loadout.make(&mut w.ecs)
    }

    /// Return whether this form can be spawned on given type of area.
    pub fn can_spawn(&self, spec: &AreaSpec) -> bool {
        // Special check, zero commonness forms never spawn.
        if self.commonness == 0 {
            return false;
        }

        self.min_depth <= spec.depth && self.biome.intersects(spec.biome)
    }
}

thread_local!(static FORMS: Vec<Rc<Form>> = init_forms());

fn init_forms() -> Vec<Rc<Form>> {
    vec![
        Rc::new(Form::mob("player",     Brush::Human,   AZURE,      10, &[Hands]).common(0)),
        Rc::new(Form::mob("dreg",       Brush::Dreg,    OLIVE,      1,  &[Hands])),
        Rc::new(Form::mob("snake",      Brush::Snake,   GREEN,      1,  &[]).biome(Overland)),
    ]
}

/// Perform operations on the collection of entity forms.
pub fn with_forms<F, U>(f: F) -> U
    where F: FnOnce(&Vec<Rc<Form>>) -> U + 'static + Sized
{
    FORMS.with(|v| f(v))
}

/// Generate a spawn probability weighted list of forms of a specific category
/// that can spawn in the desired type of area.
pub fn form_distribution(spec: &AreaSpec,
                         category: FormType)
                         -> Vec<Weighted<Rc<Form>>> {
    let spec = *spec; // Make the borrow checker happy.
    with_forms(move |v| {
        v.iter()
         .filter(|f| f.can_spawn(&spec) && f.category == category)
         .map(|f| {
             Weighted {
                 weight: f.commonness,
                 item: f.clone(),
             }
         })
         .collect()
    })
}

/// Memoizing entity spawner construct.
pub struct Spawner(HashMap<(AreaSpec, FormType), Vec<Weighted<Rc<Form>>>>);

impl Spawner {
    pub fn new() -> Spawner {
        Spawner(HashMap::new())
    }

    pub fn spawn<R: Rng>(&mut self,
                         rng: &mut R,
                         spec: &AreaSpec,
                         category: FormType)
                         -> Option<Rc<Form>> {
        // XXX: Bit ineffective making a copy of the spec here.
        let key = (*spec, category);
        if !self.0.contains_key(&key) {
            self.0.insert(key, form_distribution(spec, category));
        }

        let items = self.0.get_mut(&key).unwrap();
        if items.len() == 0 {
            None
        } else {
            Some(WeightedChoice::new(items).ind_sample(rng))
        }
    }
}
