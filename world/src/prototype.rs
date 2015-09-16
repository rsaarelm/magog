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
use content::Brush;
use world;

#[derive(Copy, Clone)]
pub struct Prototype {
    pub target: Entity
}

impl Prototype {
    pub fn new(parent: Option<Entity>) -> Prototype {
        Prototype {
            target: world::with_mut(|w| {
                let e = w.old_ecs.new_entity(parent);
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
        .c(Desc::new("player", Brush::Human, AZURE))
        .c(Stats::new(10, &[Hands]).mana(5))
        .c(MapMemory::new())
        ;

    // Enemies
    Prototype::new(Some(base_mob))
        .c(Desc::new("dreg", Brush::Dreg, OLIVE))
        .c(Stats::new(1, &[Hands]))
        .c(Spawn::new(SpawnType::Creature))
        ;

    Prototype::new(Some(base_mob))
        .c(Desc::new("snake", Brush::Snake, GREEN))
        .c(Stats::new(1, &[]))
        .c(Spawn::new(SpawnType::Creature).biome(Overland))
        ;

    Prototype::new(Some(base_mob))
        .c(Desc::new("ooze", Brush::Ooze, LIGHTSEAGREEN))
        .c(Stats::new(3, &[Slow, Deathsplosion]))
        .c(Spawn::new(SpawnType::Creature).biome(Dungeon).depth(3))
        ;

    Prototype::new(Some(base_mob))
        .c(Desc::new("ogre", Brush::Ogre, DARKSLATEGRAY))
        .c(Stats::new(6, &[Hands]))
        .c(Spawn::new(SpawnType::Creature).depth(5))
        ;

    Prototype::new(Some(base_mob))
        .c(Desc::new("wraith", Brush::Wraith, HOTPINK))
        .c(Stats::new(8, &[Hands]))
        .c(Spawn::new(SpawnType::Creature).biome(Dungeon).depth(6))
        ;

    Prototype::new(Some(base_mob))
        .c(Desc::new("octopus", Brush::Octopus, DARKTURQUOISE))
        .c(Stats::new(10, &[]))
        .c(Spawn::new(SpawnType::Creature).depth(7))
        ;

    Prototype::new(Some(base_mob))
        .c(Desc::new("efreet", Brush::Efreet, ORANGE))
        .c(Stats::new(12, &[]))
        .c(Spawn::new(SpawnType::Creature).depth(8))
        ;

    Prototype::new(Some(base_mob))
        .c(Desc::new("serpent", Brush::Serpent, CORAL))
        .c(Stats::new(15, &[]))
        .c(Spawn::new(SpawnType::Creature).biome(Dungeon).depth(9))
        ;

    // Items
    Prototype::new(None)
        .c(Desc::new("heart", Brush::Health, RED))
        .c(Spawn::new(SpawnType::Consumable).commonness(50))
        .c(Item { item_type: ItemType::Instant, ability: Ability::HealInstant(2) })
        ;

    Prototype::new(None)
        .c(Desc::new("sword", Brush::Sword, GAINSBORO))
        .c(Spawn::new(SpawnType::Equipment).commonness(100))
        .c(Stats::new(0, &[]).attack(5).mana(-3))
        .c(Item { item_type: ItemType::MeleeWeapon, ability: Ability::Multi(vec![]) })
        ;

    Prototype::new(None)
        .c(Desc::new("throwing knives", Brush::Knives, GAINSBORO))
        .c(Spawn::new(SpawnType::Equipment).commonness(500))
        .c(Stats::new(0, &[]).ranged_range(5).ranged_power(5))
        .c(Item { item_type: ItemType::RangedWeapon, ability: Ability::Multi(vec![]) })
        ;

    Prototype::new(None)
        .c(Desc::new("helmet", Brush::Helmet, GAINSBORO))
        .c(Spawn::new(SpawnType::Equipment).commonness(100))
        .c(Stats::new(0, &[]).protection(2).mana(-1))
        .c(Item { item_type: ItemType::Helmet, ability: Ability::Multi(vec![]) })
        ;

    Prototype::new(None)
        .c(Desc::new("armor", Brush::Armor, GAINSBORO))
        .c(Spawn::new(SpawnType::Equipment).commonness(100))
        .c(Stats::new(0, &[]).protection(5).mana(-3))
        .c(Item { item_type: ItemType::Armor, ability: Ability::Multi(vec![]) })
        ;
}
