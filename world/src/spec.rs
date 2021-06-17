//! Data for generating game entities.

use crate::{
    ai::{Brain, ShoutType},
    desc::{Desc, Icon},
    item::ItemType,
    item::{Item, Stacking},
    sector::Biome,
    stats::{Health, Intrinsic, Stats, StatsComponent, Statuses},
    world::Loadout,
    Anim, Distribution, ExternalEntity, Rng,
};
use lazy_static::lazy_static;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

pub trait Spec: Distribution<ExternalEntity> + Sync + Send {
    /// How rare is this spec?
    ///
    /// Rarity is the inverse of spawn probability. Rarity zero means the spec will never spawn
    /// during random sampling.
    fn rarity(&self) -> f32;

    /// What's the smallest depth where this spec can spawn?
    ///
    /// More powerful items and entities should only start spawning at lower depths.
    fn min_depth(&self) -> i32;

    /// What biomes can this spawn in
    fn habitat(&self) -> u64;

    fn name(&self) -> &str;

    /// Return base id of entity without pluralization
    fn id(&self) -> &str {
        let name = self.name();
        if let Some(offset) = name.find('|') {
            &name[..offset]
        } else {
            name
        }
    }
}

const EVERYWHERE: u64 = 0xffff_ffff_ffff_ffff;
const DUNGEON: u64 = 1 << Biome::Dungeon as u64;
const TEMPERATE: u64 = (1 << Biome::Grassland as u64) | (1 << Biome::Forest as u64);
const ARID: u64 = (1 << Biome::Desert as u64) | (1 << Biome::Mountain as u64);
const URBAN: u64 = 1 << Biome::City as u64;

#[derive(Debug)]
pub struct MobSpec {
    name: String,
    icon: Icon,
    depth: i32,
    rarity: f32,
    habitat: u64,
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
            habitat: EVERYWHERE,
            power: 0,
            intrinsics: Vec::new(),
            shout: ShoutType::Silent,
        }
    }
}

impl Distribution<ExternalEntity> for MobSpec {
    fn sample(&self, _: &mut Rng) -> ExternalEntity {
        ExternalEntity::new(
            Loadout::default()
                .c(StatsComponent::new(Stats::new(
                    self.power,
                    &self.intrinsics,
                )))
                .c(Desc::new(&self.name, self.icon))
                .c(Brain::enemy().shout(self.shout))
                .c(Anim::default())
                .c(Health::default())
                .c(Statuses::default()),
        )
    }
}

impl Spec for MobSpec {
    fn rarity(&self) -> f32 { self.rarity }
    fn min_depth(&self) -> i32 { self.depth }
    fn habitat(&self) -> u64 { self.habitat }
    fn name(&self) -> &str { &self.name }
}

#[derive(Debug)]
pub struct ItemSpec {
    name: String,
    icon: Icon,
    depth: i32,
    rarity: f32,
    habitat: u64,
    item_type: ItemType,
    power: i32,
    armor: i32,
    attack: i32,
    defense: i32,
    intrinsics: Vec<Intrinsic>,
    stacks: bool,
}

impl Default for ItemSpec {
    fn default() -> Self {
        ItemSpec {
            name: "N/A".into(),
            icon: Icon::Sword,
            depth: 0,
            rarity: 1.0,
            // As a rule, you don't find items laying around in the wilderness.
            habitat: DUNGEON,
            item_type: ItemType::MeleeWeapon,
            power: 0,
            armor: 0,
            attack: 0,
            defense: 0,
            intrinsics: Vec::new(),
            stacks: false,
        }
    }
}

impl Distribution<ExternalEntity> for ItemSpec {
    fn sample(&self, _: &mut Rng) -> ExternalEntity {
        let mut loadout = Loadout::default()
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
            });
        if self.stacks {
            loadout = loadout.c(Stacking::default());
        }
        ExternalEntity::new(loadout)
    }
}

impl Spec for ItemSpec {
    fn rarity(&self) -> f32 { self.rarity }
    fn min_depth(&self) -> i32 { self.depth }
    fn habitat(&self) -> u64 { self.habitat }
    fn name(&self) -> &str { &self.name }
}

macro_rules! specs {
    {$($item:expr,)+}
    =>
    {
        lazy_static! {
            pub static ref SPECS: BTreeMap<EntitySpawn, Arc<dyn Spec>> = {
                let mut ret: BTreeMap<EntitySpawn, Arc<dyn Spec>> = BTreeMap::new();
                $(ret.insert(EntitySpawn($item.id().to_string()), Arc::new($item));)+
                ret
            };
        }
    }
}

pub fn iter_specs() -> impl Iterator<Item = Arc<dyn Spec + 'static>> { SPECS.values().cloned() }

use self::Intrinsic::*;
use self::ShoutType::*;
use crate::effect::Ability::*;
use crate::Icon as I;
use crate::ItemType::*;

specs! {
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
        habitat: DUNGEON,
        power: 2,
        intrinsics: vec![Hands],
        shout: Shout,
        ..d()
    },
    MobSpec {
        name: "snake".into(),
        icon: I::Snake,
        habitat: DUNGEON | TEMPERATE | ARID | URBAN,
        power: 1,
        shout: Hiss,
        ..d()
    },
    MobSpec {
        name: "ooze".into(),
        icon: I::Ooze,
        depth: 1,
        habitat: DUNGEON,
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
        name: "octopus|octopi".into(),
        icon: I::Octopus,
        depth: 2,
        habitat: DUNGEON | TEMPERATE,
        power: 5,
        intrinsics: vec![Hands],
        ..d()
    },
    MobSpec {
        name: "ogre".into(),
        icon: I::Ogre,
        depth: 5,
        rarity: 4.0,
        habitat: DUNGEON | ARID,
        power: 7,
        intrinsics: vec![Hands],
        shout: Shout,
        ..d()
    },
    MobSpec {
        name: "wraith".into(),
        icon: I::Wraith,
        depth: 6,
        habitat: DUNGEON,
        rarity: 6.0,
        power: 10,
        intrinsics: vec![Hands],
        ..d()
    },
    MobSpec {
        name: "efreeti|efreet".into(),
        icon: I::Efreet,
        depth: 7,
        habitat: DUNGEON,
        rarity: 8.0,
        power: 14,
        intrinsics: vec![Hands],
        ..d()
    },
    MobSpec {
        name: "serpent".into(),
        icon: I::Serpent,
        depth: 9,
        habitat: DUNGEON,
        rarity: 10.0,
        power: 20,
        shout: Hiss,
        ..d()
    },
    MobSpec {
        name: "bear".into(),
        icon: I::Bear,
        depth: 2,
        habitat: TEMPERATE,
        power: 7,
        shout: Roar,
        ..d()
    },
    MobSpec {
        name: "spider".into(),
        icon: I::Spider,
        depth: 4,
        habitat: DUNGEON | ARID,
        power: 20,
        rarity: 10.0,
        shout: Hiss,
        ..d()
    },
    MobSpec {
        name: "totem guardian".into(),
        icon: I::TotemGuardian,
        depth: 4,
        habitat: DUNGEON,
        rarity: 5.0,
        power: 5,
        intrinsics: vec![Hands],
        ..d()
    },
    MobSpec {
        name: "bat".into(),
        icon: I::Bat,
        habitat: DUNGEON | TEMPERATE | URBAN | ARID,
        power: 1,
        intrinsics: vec![Hyperactive],
        ..d()
    },
    MobSpec {
        name: "centaur".into(),
        icon: I::Centaur,
        habitat: DUNGEON | TEMPERATE | ARID,
        depth: 4,
        power: 5,
        rarity: 3.0,
        shout: Shout,
        intrinsics: vec![Hands],
        ..d()
    },
    MobSpec {
        name: "armored construct".into(),
        icon: I::ArmorConstruct,
        habitat: DUNGEON,
        depth: 6,
        power: 8,
        rarity: 5.0,
        intrinsics: vec![Hands],
        ..d()
    },
    MobSpec {
        name: "curse skull".into(),
        icon: I::CurseSkull,
        habitat: DUNGEON,
        depth: 6,
        power: 8,
        rarity: 5.0,
        shout: Shout,
        ..d()
    },
    MobSpec {
        name: "gelatinous cube".into(),
        icon: I::GelatinousCube,
        habitat: DUNGEON,
        depth: 3,
        power: 5,
        rarity: 3.0,
        shout: Gurgle,
        ..d()
    },
    MobSpec {
        name: "crocodile".into(),
        icon: I::Crocodile,
        habitat: TEMPERATE | ARID,
        depth: 3,
        power: 5,
        rarity: 2.0,
        shout: Roar,
        ..d()
    },
    MobSpec {
        name: "wisp".into(),
        icon: I::Wisp,
        habitat: DUNGEON,
        depth: 5,
        power: 5,
        rarity: 10.0,
        ..d()
    },
    MobSpec {
        name: "vortex|vortices".into(),
        icon: I::Vortex,
        habitat: DUNGEON,
        depth: 8,
        power: 10,
        rarity: 10.0,
        ..d()
    },
    MobSpec {
        name: "moloch".into(),
        icon: I::Moloch,
        habitat: DUNGEON,
        depth: 10,
        power: 50,
        rarity: 20.0,
        ..d()
    },
    MobSpec {
        name: "lizardman|lizardmen".into(),
        icon: I::Lizardman,
        habitat: TEMPERATE | ARID,
        power: 2,
        rarity: 2.0,
        shout: Shout,
        intrinsics: vec![Hands],
        ..d()
    },
    MobSpec {
        name: "centipede".into(),
        icon: I::Centipede,
        habitat: DUNGEON,
        depth: 8,
        power: 10,
        rarity: 8.0,
        ..d()
    },
    MobSpec {
        name: "floating eye".into(),
        icon: I::FloatingEye,
        habitat: DUNGEON,
        power: 2,
        ..d()
    },
    MobSpec {
        name: "eye horror".into(),
        icon: I::EyeHorror,
        habitat: DUNGEON,
        depth: 6,
        power: 8,
        rarity: 3.0,
        intrinsics: vec![Deathsplosion],
        ..d()
    },
    MobSpec {
        name: "dog".into(),
        icon: I::Dog,
        habitat: DUNGEON,
        power: 3,
        rarity: 10.0,
        ..d()
    },
    MobSpec {
        name: "cat".into(),
        icon: I::Cat,
        habitat: DUNGEON,
        power: 3,
        rarity: 10.0,
        ..d()
    },
    MobSpec {
        name: "rat".into(),
        icon: I::Rat,
        habitat: DUNGEON | TEMPERATE | ARID,
        power: 1,
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
        name: "wand of fireball|wands of fireball".into(),
        icon: I::Wand1,
        power: 5,
        item_type: TargetedUsable(Fireball),
        rarity: 10.0,
        depth: 3,
        ..d()
    },
    ItemSpec {
        name: "wand of confusion|wands of confusion".into(),
        icon: I::Wand2,
        power: 5,
        item_type: TargetedUsable(Confuse),
        rarity: 10.0,
        armor: 5,
        ..d()
    },
    ItemSpec {
        name: "scroll of lightning|scrolls of lightning".into(),
        icon: I::Scroll1,
        power: 1,
        item_type: UntargetedUsable(LightningBolt),
        stacks: true,
        ..d()
    },
}

/// String that's guaranteed to describe an entity spawn.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct EntitySpawn(String);

impl fmt::Display for EntitySpawn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}

lazy_static! {
    pub static ref PLAYER_SPAWN: EntitySpawn = EntitySpawn("player".to_string());
}

#[derive(Debug)]
pub struct SpawnError(String);

impl fmt::Display for SpawnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
        EntitySpawn::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Distribution<ExternalEntity> for EntitySpawn {
    fn sample(&self, rng: &mut Rng) -> ExternalEntity {
        SPECS
            .get(self)
            .unwrap_or_else(|| panic!("EntitySpawn {:?} not found in spec database", self))
            .sample(rng)
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
