use std::default::Default;
use calx::color::*;
use content::{SpawnType};
use ecs::{Component, ComponentAccess};
use entity::{Entity};
use components::{Spawn, IsPrototype};
use components::{Desc, MapMemory, Health};
use components::{Brain, BrainState, Alignment};
use components::{Item};
use item::{ItemType};
use stats::{Stats};
use stats::Intrinsic::*;
use ability::Ability;
use content::Biome::*;
use world;

#[derive(Copy, Clone)]
pub struct Prototype {
    pub target: Entity
}

impl Prototype {
    pub fn new(parent: Option<Entity>) -> Prototype {
        Prototype {
            target: world::with_mut(|w| {
                let e = w.ecs.new_entity(parent);
                w.prototypes_mut().insert(e, IsPrototype);
                e
            }),
        }
    }

    /// Add a component to the prototype.
    pub fn c<C: Component>(self, component: C) -> Prototype {
        component.add_to(self.target);
        self
    }
}

/// Only call at world init!
pub fn init() {
    let base_mob = Prototype::new(None)
        .c(Brain { state: BrainState::Asleep, alignment: Alignment::Evil })
        .c({let h: Health = Default::default(); h})
        .target;

    // Init the prototypes

    // Player
    Prototype::new(Some(base_mob))
        .c(Brain { state: BrainState::PlayerControl, alignment: Alignment::Good })
        .c(Desc::new("player", 51, AZURE))
        .c(Stats::new(10, &[Hands]).mana(5))
        .c(MapMemory::new())
        ;

    // Enemies
    Prototype::new(Some(base_mob))
        .c(Desc::new("dreg", 72, OLIVE))
        .c(Stats::new(1, &[Hands]))
        .c(Spawn::new(SpawnType::Creature))
        ;

    Prototype::new(Some(base_mob))
        .c(Desc::new("snake", 71, GREEN))
        .c(Stats::new(1, &[]))
        .c(Spawn::new(SpawnType::Creature).biome(Overland))
        ;

    Prototype::new(Some(base_mob))
        .c(Desc::new("ooze", 77, LIGHTSEAGREEN))
        .c(Stats::new(3, &[Slow, Deathsplosion]))
        .c(Spawn::new(SpawnType::Creature).biome(Dungeon).depth(3))
        ;

    Prototype::new(Some(base_mob))
        .c(Desc::new("ogre", 73, DARKSLATEGRAY))
        .c(Stats::new(6, &[Hands]))
        .c(Spawn::new(SpawnType::Creature).depth(5))
        ;

    Prototype::new(Some(base_mob))
        .c(Desc::new("wraith", 74, HOTPINK))
        .c(Stats::new(8, &[Hands]))
        .c(Spawn::new(SpawnType::Creature).biome(Dungeon).depth(6))
        ;

    Prototype::new(Some(base_mob))
        .c(Desc::new("octopus", 63, DARKTURQUOISE))
        .c(Stats::new(10, &[]))
        .c(Spawn::new(SpawnType::Creature).depth(7))
        ;

    Prototype::new(Some(base_mob))
        .c(Desc::new("efreet", 78, ORANGE))
        .c(Stats::new(12, &[]))
        .c(Spawn::new(SpawnType::Creature).depth(8))
        ;

    Prototype::new(Some(base_mob))
        .c(Desc::new("serpent", 94, CORAL))
        .c(Stats::new(15, &[]))
        .c(Spawn::new(SpawnType::Creature).biome(Dungeon).depth(9))
        ;

    // Items
    Prototype::new(None)
        .c(Desc::new("heart", 89, RED))
        .c(Spawn::new(SpawnType::Consumable).commonness(50))
        .c(Item { item_type: ItemType::Instant, ability: Ability::HealInstant(2) })
        ;

    Prototype::new(None)
        .c(Desc::new("sword", 84, GAINSBORO))
        .c(Spawn::new(SpawnType::Equipment).commonness(100))
        .c(Stats::new(0, &[]).attack(5).mana(-3))
        .c(Item { item_type: ItemType::MeleeWeapon, ability: Ability::Multi(vec![]) })
        ;

    Prototype::new(None)
        .c(Desc::new("throwing knives", 90, GAINSBORO))
        .c(Spawn::new(SpawnType::Equipment).commonness(500))
        .c(Stats::new(0, &[]).ranged_range(5).ranged_power(5))
        .c(Item { item_type: ItemType::RangedWeapon, ability: Ability::Multi(vec![]) })
        ;

    Prototype::new(None)
        .c(Desc::new("helmet", 85, GAINSBORO))
        .c(Spawn::new(SpawnType::Equipment).commonness(100))
        .c(Stats::new(0, &[]).protection(2).mana(-1))
        .c(Item { item_type: ItemType::Helmet, ability: Ability::Multi(vec![]) })
        ;

    Prototype::new(None)
        .c(Desc::new("armor", 91, GAINSBORO))
        .c(Spawn::new(SpawnType::Equipment).commonness(100))
        .c(Stats::new(0, &[]).protection(5).mana(-3))
        .c(Item { item_type: ItemType::Armor, ability: Ability::Multi(vec![]) })
        ;
}
