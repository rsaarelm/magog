use calx::WeightedChoice;
use components::{Icon, Brain, Desc, Health, Item, MapMemory, ShoutType, StatsComponent, Statuses};
use item::ItemType;
use rand::Rng;
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

/// Sample a weighted choice from a filtered Form selection.
pub fn rand<R: Rng>(rng: &mut R, selection: &[&'static Form]) -> Option<&'static Form> {
    selection
        .weighted_choice(rng, |item| if item.rarity == 0.0 {
            0.0
        } else {
            1.0 / item.rarity
        })
        .cloned()
}

impl Form {
    pub fn named(name: &str) -> Option<&'static Form> {
        FORMS.iter().find(|x| x.name() == Some(name))
    }

    pub fn filter<F: Fn(&Form) -> bool>(p: F) -> Vec<&'static Form> {
        FORMS.iter().filter(|&x| p(x)).collect()
    }

    /// Create a standard form for a living creature.
    pub fn mob(name: &str, brush: Icon, power: i32, intrinsics: &[Intrinsic]) -> Form {
        Form {
            rarity: 1.0,
            min_depth: 0,
            // category: FormType::Creature,
            loadout: Loadout::new()
                .c(StatsComponent::new(Stats::new(power, intrinsics)))
                .c(Desc::new(name, brush))
                .c(Brain::enemy())
                .c(Health::new())
                .c(Statuses::new()),
        }
    }

    pub fn item(name: &str, brush: Icon, power: i32, item_type: ItemType) -> Form {
        Form {
            rarity: 1.0,
            min_depth: 0,
            loadout: Loadout::new()
                .c(StatsComponent::new(Stats::new(power, &[])))
                .c(Desc::new(name, brush))
                .c(Item {
                    item_type,
                    charges: 1,
                }),
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

    pub fn folk(mut self) -> Form {
        self.loadout.brain.as_mut().expect("Must be mob").shout = ShoutType::Shout;
        self.loadout.stats.as_mut().unwrap().base.add_intrinsic(Intrinsic::Hands);
        self
    }

    pub fn reptile(mut self) -> Form {
        self.loadout.brain.as_mut().expect("Must be mob").shout = ShoutType::Hiss;
        self
    }

    pub fn attack(mut self, bonus: i32) -> Form {
        self.loadout.stats.as_mut().unwrap().base.attack += bonus;
        self
    }

    pub fn defense(mut self, bonus: i32) -> Form {
        self.loadout.stats.as_mut().unwrap().base.defense += bonus;
        self
    }

    pub fn armor(mut self, bonus: i32) -> Form {
        self.loadout.stats.as_mut().unwrap().base.armor += bonus;
        self
    }


    /// Return the name of the form if it has one.
    pub fn name(&self) -> Option<&str> {
        match self.loadout.desc {
            Some(ref desc) => Some(&desc.name[..]),
            None => None,
        }
    }

    pub fn is_item(&self) -> bool { self.loadout.item.is_some() }

    pub fn is_mob(&self) -> bool { self.loadout.brain.is_some() }

    pub fn at_depth(&self, depth: i32) -> bool { self.min_depth <= depth }

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
        Form::mob("dreg",       Icon::Dreg,       2,  &[]).folk(),
        Form::mob("snake",      Icon::Snake,      1,  &[]).reptile(),
        Form::mob("ooze",       Icon::Ooze,       3,  &[]).depth(1),
        Form::mob("bug",        Icon::Bug,        2,  &[]).depth(2).rarity(10.0),
        Form::mob("octopus",    Icon::Octopus,    5,  &[Hands]).depth(3),
        Form::mob("ogre",       Icon::Ogre,       7,  &[]).folk().depth(4),
        Form::mob("wraith",     Icon::Wraith,     10, &[Hands]).depth(5),
        Form::mob("efreet",     Icon::Efreet,     14, &[Hands]).depth(7),
        Form::mob("serpent",    Icon::Serpent,    20, &[]).depth(8).rarity(5.0).reptile(),

        Form::item("sword",     Icon::Sword,     0,  ItemType::MeleeWeapon).rarity(10.0).attack(6),
        Form::item("helmet",    Icon::Helmet,    0,  ItemType::Helmet).rarity(10.0).armor(2),
        Form::item("armor",     Icon::Armor,     0,  ItemType::Armor).rarity(10.0).armor(5),
        Form::item("wand of fireball",    Icon::Wand1,     5,  ItemType::TargetedUsable(Fireball)).depth(3),
        Form::item("wand of confusion",   Icon::Wand2,     5,  ItemType::TargetedUsable(Confuse)),
        Form::item("scroll of lightning", Icon::Scroll1,   1,  ItemType::UntargetedUsable(Lightning)),
        ]
    };
}
