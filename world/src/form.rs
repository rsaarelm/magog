use calx_ecs::{Entity};
use calx::color::*;
use calx::{Rgba};
use content::{Biome, FormType, Brush};
use content::Biome::*;
use world::{Component};
use stats::{Stats, Intrinsic};
use stats::Intrinsic::*;
use components::{Desc, Brain, Health};
use world::{World};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Forms {
    Player,
    Dreg,
    Snake,
    Ooze,
}

/// Forms are the prototypes for the entities you create.
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

    // Stats needs to be kept outside the trait object bag in form, since it
    // will be modified by fluent operations, and once something is in the
    // loadout bag, it's stuck behind the opaque trait object interface.
    stats: Stats,

    /// Actual components to set up the thing.
    form: Vec<Box<Component>>,
}

impl Form {
    /// Create a standard form for a living creature.
    pub fn mob(
        name: &str,
        icon: Brush,
        color: Rgba,
        power: i32,
        intrinsics: &[Intrinsic]) -> Form {
        Form {
            biome: Anywhere,
            commonness: 1000,
            min_depth: 0,
            category: FormType::Creature,
            stats: Stats::new(power, intrinsics),
            form: loadout! {
                Desc::new(name, icon, color),
                Brain::enemy(),
                Health::new()
            },
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
        let ret = w.ecs.make();
        for comp in self.form.iter() {
            comp.add_to(&mut w.ecs, ret);
        }

        self.stats.add_to(&mut w.ecs, ret);

        ret
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
