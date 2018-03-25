use calx::WeightedChoice;
use components::{Brain, Desc, Health, Icon, Item, ShoutType, StatsComponent, Statuses};
use item::ItemType;
use rand::Rng;
use stats::{Intrinsic, Stats};
use world::Loadout;

pub trait Spec {
    /// Generate an entity loadout for this spec.
    fn generate<R: Rng>(&self, &mut R) -> Loadout;

    /// How rare is this spec?
    ///
    /// Rarity is the inverse of spawn probability. Rarity zero means the spec will never spawn
    /// during random sampling.
    fn rarity(&self) -> f32;

    /// What's the smallest depth where this spec can spawn?
    ///
    /// More powerful items and entities should only start spawning at lower depths.
    fn min_depth(&self) -> i32;
}

#[derive(Debug)]
pub struct MobSpec {
    name: String,
    icon: Icon,
    depth: i32,
    rarity: f32,
    power: i32,
    intrinsics: Vec<Intrinsic>,
    shout: ShoutType,
}

impl Default for MobSpec {
    fn default() -> Self {
        MobSpec {
            name: "N/A".into(),
            icon: Icon::Player,
            depth: 0,
            rarity: 1.0,
            power: 0,
            intrinsics: Vec::new(),
            shout: ShoutType::Silent,
        }
    }
}

impl Spec for MobSpec {
    fn generate<R: Rng>(&self, _: &mut R) -> Loadout {
        Loadout::new()
            .c(StatsComponent::new(Stats::new(
                self.power,
                &self.intrinsics,
            )))
            .c(Desc::new(&self.name, self.icon))
            .c(Brain::enemy())
            .c(Health::new())
            .c(Statuses::new())
    }

    fn rarity(&self) -> f32 { self.rarity }
    fn min_depth(&self) -> i32 { self.depth }
}

#[derive(Debug)]
pub struct ItemSpec {
    name: String,
    icon: Icon,
    depth: i32,
    rarity: f32,
    item_type: ItemType,
    power: i32,
    armor: i32,
    attack: i32,
    defense: i32,
    intrinsics: Vec<Intrinsic>,
}

impl Default for ItemSpec {
    fn default() -> Self {
        ItemSpec {
            name: "N/A".into(),
            icon: Icon::Sword,
            depth: 0,
            rarity: 1.0,
            item_type: ItemType::MeleeWeapon,
            power: 0,
            armor: 0,
            attack: 0,
            defense: 0,
            intrinsics: Vec::new(),
        }
    }
}

impl Spec for ItemSpec {
    fn generate<R: Rng>(&self, _: &mut R) -> Loadout {
        Loadout::new()
            .c(Desc::new(&self.name, self.icon))
            .c(StatsComponent::new(
                Stats::new(self.power, &self.intrinsics)
                    .armor(self.armor)
                    .attack(self.attack)
                    .defense(self.defense),
            ))
            .c(Item {
                item_type: self.item_type,
                charges: 1,
            })
    }

    fn rarity(&self) -> f32 { self.rarity }
    fn min_depth(&self) -> i32 { self.depth }
}

lazy_static! {
    pub static ref MOB_SPECS: Vec<MobSpec> = {
        use self::Intrinsic::*;
        use Icon::*;
        use self::ShoutType::*;

        vec![
            MobSpec {
                name: "player".into(),
                icon: Player,
                rarity: 0.0,
                power: 10,
                intrinsics: vec![Hands],
                shout: Shout,
                ..d()
            },

            MobSpec {
                name: "dreg".into(),
                icon: Dreg,
                power: 2,
                intrinsics: vec![Hands],
                shout: Shout,
                ..d()
            },

            MobSpec {
                name: "snake".into(),
                icon: Snake,
                power: 1,
                shout: Hiss,
                ..d()
            },

            MobSpec {
                name: "ooze".into(),
                icon: Ooze,
                depth: 1,
                power: 3,
                shout: Gurgle,
                ..d()
            },

            MobSpec {
                name: "bug".into(),
                icon: Bug,
                depth: 2,
                rarity: 10.0,
                power: 2,
                ..d()
            },

            MobSpec {
                name: "octopus".into(),
                icon: Octopus,
                depth: 2,
                power: 5,
                intrinsics: vec![Hands],
                ..d()
            },

            MobSpec {
                name: "ogre".into(),
                icon: Ogre,
                depth: 4,
                rarity: 4.0,
                power: 7,
                intrinsics: vec![Hands],
                shout: Shout,
                ..d()
            },

            MobSpec {
                name: "wraith".into(),
                icon: Wraith,
                depth: 5,
                rarity: 6.0,
                power: 10,
                intrinsics: vec![Hands],
                ..d()
            },

            MobSpec {
                name: "efreet".into(),
                icon: Efreet,
                depth: 7,
                rarity: 8.0,
                power: 14,
                intrinsics: vec![Hands],
                ..d()
            },

            MobSpec {
                name: "serpent".into(),
                icon: Serpent,
                depth: 9,
                rarity: 10.0,
                power: 20,
                shout: Hiss,
                ..d()
            },
        ]
    };

    pub static ref ITEM_SPECS: Vec<ItemSpec> = {
        use ItemType::*;
        use item::MagicEffect::*;
        use Icon as I;

        vec![
            ItemSpec {
                name: "sword".into(),
                icon: I::Sword,
                item_type: MeleeWeapon,
                rarity: 10.0,
                attack: 6,
                ..d()
            },

            ItemSpec {
                name: "helmet".into(),
                icon: I::Helmet,
                item_type: Helmet,
                rarity: 10.0,
                armor: 2,
                ..d()
            },

            ItemSpec {
                name: "armor".into(),
                icon: I::Armor,
                item_type: Armor,
                rarity: 10.0,
                armor: 5,
                ..d()
            },

            ItemSpec {
                name: "wand of fireball".into(),
                icon: I::Wand1,
                power: 5,
                item_type: TargetedUsable(Fireball),
                rarity: 10.0,
                depth: 3,
                ..d()
            },

            ItemSpec {
                name: "wand of confusion".into(),
                icon: I::Wand2,
                power: 5,
                item_type: TargetedUsable(Confuse),
                rarity: 10.0,
                armor: 5,
                ..d()
            },

            ItemSpec {
                name: "scroll of lightning".into(),
                icon: I::Scroll1,
                power: 1,
                item_type: UntargetedUsable(Lightning),
                ..d()
            },
        ]
    };
}

/// Sample a random spec from a list.
pub fn pick<S: Spec, R: Rng>(rng: &mut R, depth: i32, specs: &[S]) -> Option<Loadout> {
    specs
        .weighted_choice(rng, |item| {
            if item.rarity() == 0.0 || item.min_depth() > depth {
                0.0
            } else {
                1.0 / item.rarity()
            }
        })
        .map(|s| s.generate(rng))
}

pub fn named<R: Rng>(rng: &mut R, name: &str) -> Option<Loadout> {
    if let Some(spec) = MOB_SPECS.iter().find(|x| &x.name == name) {
        return Some(spec.generate(rng));
    }

    if let Some(spec) = ITEM_SPECS.iter().find(|x| &x.name == name) {
        return Some(spec.generate(rng));
    }

    None
}

pub fn is_named(name: &str) -> bool {
    use rand::StdRng;
    named(&mut StdRng::new().unwrap(), name).is_some()
}

// Helpers for data conciseness.

fn d<T: Default>() -> T { Default::default() }
