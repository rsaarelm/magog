use crate::components::{
    Anim, Brain, Desc, Health, Icon, Item, ShoutType, StatsComponent, Statuses,
};
use crate::item::ItemType;
use crate::stats::{Intrinsic, Stats};
use crate::world::Loadout;
use crate::{Distribution, Rng};
use serde;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

pub trait Spec: Distribution<Loadout> + Sync + Send {
    /// How rare is this spec?
    ///
    /// Rarity is the inverse of spawn probability. Rarity zero means the spec will never spawn
    /// during random sampling.
    fn rarity(&self) -> f32;

    /// What's the smallest depth where this spec can spawn?
    ///
    /// More powerful items and entities should only start spawning at lower depths.
    fn min_depth(&self) -> i32;

    fn name(&self) -> &str;
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

impl Distribution<Loadout> for MobSpec {
    fn sample(&self, _: &mut Rng) -> Loadout {
        Loadout::new()
            .c(StatsComponent::new(Stats::new(
                self.power,
                &self.intrinsics,
            ))).c(Desc::new(&self.name, self.icon))
            .c(Brain::enemy())
            .c(Anim::default())
            .c(Health::default())
            .c(Statuses::default())
    }
}

impl Spec for MobSpec {
    fn rarity(&self) -> f32 { self.rarity }
    fn min_depth(&self) -> i32 { self.depth }
    fn name(&self) -> &str { &self.name }
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

impl Distribution<Loadout> for ItemSpec {
    fn sample(&self, _: &mut Rng) -> Loadout {
        Loadout::new()
            .c(Desc::new(&self.name, self.icon))
            .c(StatsComponent::new(
                Stats::new(self.power, &self.intrinsics)
                    .armor(self.armor)
                    .attack(self.attack)
                    .defense(self.defense),
            )).c(Item {
                item_type: self.item_type,
                charges: 1,
            })
    }
}

impl Spec for ItemSpec {
    fn rarity(&self) -> f32 { self.rarity }
    fn min_depth(&self) -> i32 { self.depth }
    fn name(&self) -> &str { &self.name }
}

macro_rules! specs {
    {$($item:expr,)+}
    =>
    {
        lazy_static! {
            pub static ref SPECS: HashMap<EntitySpawn, Arc<dyn Spec>> = {
                let mut ret: HashMap<EntitySpawn, Arc<dyn Spec>> = HashMap::new();
                $(ret.insert(EntitySpawn($item.name().to_string()), Arc::new($item));)+
                ret
            };
        }
    }
}

pub fn iter_specs() -> impl Iterator<Item = Arc<dyn Spec + 'static>> { SPECS.values().cloned() }

use self::Intrinsic::*;
use self::ShoutType::*;
use crate::item::MagicEffect::*;
use crate::Icon as I;
use crate::ItemType::*;

specs!{
    // Mobs
    MobSpec {
        name: "player".into(),
        icon: I::Player,
        rarity: 0.0,
        power: 10,
        intrinsics: vec![Hands],
        shout: Shout,
        ..d()
    },
    MobSpec {
        name: "dreg".into(),
        icon: I::Dreg,
        power: 2,
        intrinsics: vec![Hands],
        shout: Shout,
        ..d()
    },
    MobSpec {
        name: "snake".into(),
        icon: I::Snake,
        power: 1,
        shout: Hiss,
        ..d()
    },
    MobSpec {
        name: "ooze".into(),
        icon: I::Ooze,
        depth: 1,
        power: 3,
        shout: Gurgle,
        ..d()
    },
    MobSpec {
        name: "bug".into(),
        icon: I::Bug,
        depth: 2,
        rarity: 10.0,
        power: 2,
        ..d()
    },
    MobSpec {
        name: "octopus".into(),
        icon: I::Octopus,
        depth: 2,
        power: 5,
        intrinsics: vec![Hands],
        ..d()
    },
    MobSpec {
        name: "ogre".into(),
        icon: I::Ogre,
        depth: 4,
        rarity: 4.0,
        power: 7,
        intrinsics: vec![Hands],
        shout: Shout,
        ..d()
    },
    MobSpec {
        name: "wraith".into(),
        icon: I::Wraith,
        depth: 5,
        rarity: 6.0,
        power: 10,
        intrinsics: vec![Hands],
        ..d()
    },
    MobSpec {
        name: "efreet".into(),
        icon: I::Efreet,
        depth: 7,
        rarity: 8.0,
        power: 14,
        intrinsics: vec![Hands],
        ..d()
    },
    MobSpec {
        name: "serpent".into(),
        icon: I::Serpent,
        depth: 9,
        rarity: 10.0,
        power: 20,
        shout: Hiss,
        ..d()
    },


    // Items
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
}

/// String that's guaranteed to describe an entity spawn.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct EntitySpawn(String);

impl fmt::Display for EntitySpawn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}

lazy_static! {
    pub static ref PLAYER_SPAWN: EntitySpawn = EntitySpawn("player".to_string());
}

#[derive(Debug)]
pub struct SpawnError(String);

impl fmt::Display for SpawnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EntitySpawn {} not found in spec database", self.0)
    }
}

impl Error for SpawnError {}

impl FromStr for EntitySpawn {
    type Err = SpawnError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !SPECS.contains_key(&EntitySpawn(s.to_string())) {
            Err(SpawnError(s.to_string()))
        } else {
            Ok(EntitySpawn(s.to_string()))
        }
    }
}

impl serde::Serialize for EntitySpawn {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(s)
    }
}

impl<'a> serde::Deserialize<'a> for EntitySpawn {
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        let s: String = serde::Deserialize::deserialize(d)?;
        Ok(EntitySpawn::from_str(&s).map_err(serde::de::Error::custom)?)
    }
}

impl Distribution<Loadout> for EntitySpawn {
    fn sample(&self, rng: &mut Rng) -> Loadout {
        SPECS
            .get(self)
            .expect(&format!(
                "EntitySpawn {:?} not found in spec database",
                self
            )).sample(rng)
    }
}

// Helpers for data conciseness.

fn d<T: Default>() -> T { Default::default() }

#[cfg(test)]
mod test {
    #[test]
    fn test_entity_spawn_serialization() {
        use super::EntitySpawn;
        use ron;
        use std::str::FromStr;

        let example = EntitySpawn::from_str("dreg").unwrap();
        assert!(EntitySpawn::from_str("tyop txet").is_err());

        // Check roundtrip.
        let ser = ron::ser::to_string(&example).unwrap();
        assert_eq!(example, ron::de::from_str::<EntitySpawn>(&ser).unwrap());

        // Names in spec database get deserialized.
        assert_eq!(
            ron::de::from_str::<EntitySpawn>(&"\"dreg\"".to_string()).unwrap(),
            example
        );

        // Names not in database don't.
        assert!(ron::de::from_str::<EntitySpawn>(&"\"tyop txet\"".to_string()).is_err());
    }
}
