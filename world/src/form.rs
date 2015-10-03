use calx_ecs::Entity;
use calx::color::*;
use calx::Rgba;
use content::{Biome, FormType, Brush};
use content::Biome::*;
use world::Component;
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
#[derive(Clone, Debug)]
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
    /// Type of thing 
    pub category: FormType,
    /// Actual components to set up the thing.
    pub loadout: Loadout,
}

impl Form {
    /// Create a standard form for a living creature.
    pub fn mob(name: &str, icon: Brush, color: Rgba, power: i32, intrinsics: &[Intrinsic]) -> Form {
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
}

thread_local!(static FORMS: Vec<Form> = init_forms());

fn init_forms() -> Vec<Form> {
    vec![
        Form::mob("player",     Brush::Human,   AZURE,      10, &[Hands]).common(0),
        Form::mob("dreg",       Brush::Dreg,    OLIVE,      1,  &[Hands]),
        Form::mob("snake",      Brush::Snake,   GREEN,      1,  &[]).biome(Overland),
    ]
}

/// Perform operations on the collection of entity forms.
pub fn with_forms<F, U>(f: F) -> U
    where F: FnOnce(&Vec<Form>) -> U + 'static + Sized
{
    FORMS.with(|v| f(v))
}
