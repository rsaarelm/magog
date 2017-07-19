use components::{Icon, Brain, Desc, Health, Item, MapMemory};
use item::ItemType;
use stats::{Intrinsic, Stats};
use stats::Intrinsic::*;
use world::{Component, Loadout};

/// Forms are the prototypes for the entities you create.
#[derive(Clone, Debug)]
pub struct Form {
    /// Spawn probability weight is the inverse of rarity.
    ///
    /// Rarity zero is a special case that gets zero weight.
    pub rarity: f32,
    /// Minimum depth where the entity will show up.
    ///
    /// More powerful entities only start showing up in large depths.
    pub min_depth: i32,
    //    /// Type of thing.
    //    pub category: FormType,
    /// Actual components to set up the thing.
    pub loadout: Loadout,
}

impl Form {
    pub fn named(name: &str) -> Option<&'static Form> {
        FORMS.iter().find(|x| x.name() == Some(name))
    }

    /// Create a standard form for a living creature.
    pub fn mob(name: &str, brush: Icon, power: i32, intrinsics: &[Intrinsic]) -> Form {
        Form {
            rarity: 1.0,
            min_depth: 0,
            // category: FormType::Creature,
            loadout: Loadout::new()
                .c(Stats::new(power, intrinsics))
                .c(Desc::new(name, brush))
                .c(Brain::enemy())
                .c(Health::new()),
        }
    }

    pub fn item(name: &str, brush: Icon, power: i32, item_type: ItemType) -> Form {
        Form {
            rarity: 1.0,
            min_depth: 0,
            loadout: Loadout::new()
                .c(Stats::new(power, &[]))
                .c(Desc::new(name, brush))
                .c(Item { item_type }),
        }
    }

    pub fn rarity(mut self, rarity: f32) -> Form {
        self.rarity = rarity;
        self
    }

    pub fn depth(mut self, min_depth: i32) -> Form {
        self.min_depth = min_depth;
        self
    }

    pub fn player(mut self) -> Form {
        self.loadout.brain = Some(Brain::player());
        self
    }

    /// Return the name of the form if it has one.
    pub fn name(&self) -> Option<&str> {
        match self.loadout.desc {
            Some(ref desc) => Some(&desc.name[..]),
            None => None,
        }
    }

    pub fn c<C: Component>(mut self, comp: C) -> Form {
        self.loadout = self.loadout.c(comp);
        self
    }
}

lazy_static! {
    pub static ref FORMS: Vec<Form> = {
        use item::MagicEffect::*;
        vec![
        Form::mob("player",     Icon::Player,     10, &[Hands]).rarity(0.0).player()
            .c(MapMemory::default()),
        Form::mob("dreg",       Icon::Dreg,       1,  &[Hands]),
        Form::mob("snake",      Icon::Snake,      1,  &[]),

        Form::item("sword",     Icon::Sword,     10,  ItemType::MeleeWeapon),
        Form::item("wand of fireball",    Icon::Wand1,     5,  ItemType::TargetedUsable(Fireball)),
        Form::item("wand of confusion",   Icon::Wand2,     5,  ItemType::TargetedUsable(Confuse)),
        Form::item("scroll of lightning", Icon::Scroll1,   1,  ItemType::UntargetedUsable(Lightning)),
        ]
    };
}
