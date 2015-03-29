use std::default::Default;
use calx::color::*;
use super::ecs::{Component, ComponentAccess};
use super::entity::{Entity};
use super::components::{Spawn, Category, IsPrototype};
use super::components::{Desc, MapMemory, Health};
use super::components::{Brain, BrainState, Alignment};
use super::components::{Item};
use super::item::{ItemType};
use super::stats::{Stats};
use super::stats::Intrinsic::*;
use super::ability::Ability;
use super::Biome::*;
use super::world;

#[derive(Copy)]
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
}

impl<C: Component> Fn<(C,)> for Prototype {
    extern "rust-call" fn call(&self, (comp,): (C,)) -> Prototype {
        comp.add_to(self.target);
        *self
    }
}

impl<C: Component> FnMut<(C,)> for Prototype {
    extern "rust-call" fn call_mut(&mut self, (comp,): (C,)) -> Prototype {
        Fn::call(*&self, (comp,))
    }
}

impl<C: Component> FnOnce<(C,)> for Prototype {
    type Output = Prototype;

    extern "rust-call" fn call_once(self, (comp,): (C,)) -> Prototype {
        Fn::call(&self, (comp,))
    }
}

/// Only call at world init!
pub fn init() {
    let base_mob = Prototype::new(None)
        (Brain { state: BrainState::Asleep, alignment: Alignment::Evil })
        ({let h: Health = Default::default(); h})
        .target;

    // Init the prototypes

    // Player
    Prototype::new(Some(base_mob))
        (Brain { state: BrainState::PlayerControl, alignment: Alignment::Good })
        (Desc::new("player", 51, AZURE))
        (Stats::new(10, &[Hands]).mana(5))
        (MapMemory::new())
        ;

    // Enemies
    Prototype::new(Some(base_mob))
        (Desc::new("dreg", 72, OLIVE))
        (Stats::new(1, &[Hands]))
        (Spawn::new(Category::Mob))
        ;

    Prototype::new(Some(base_mob))
        (Desc::new("snake", 71, GREEN))
        (Stats::new(1, &[]))
        (Spawn::new(Category::Mob).biome(Overland))
        ;

    Prototype::new(Some(base_mob))
        (Desc::new("ooze", 77, LIGHTSEAGREEN))
        (Stats::new(3, &[]))
        (Spawn::new(Category::Mob).biome(Dungeon).depth(3))
        ;

    Prototype::new(Some(base_mob))
        (Desc::new("ogre", 73, DARKSLATEGRAY))
        (Stats::new(6, &[Hands]))
        (Spawn::new(Category::Mob).depth(5))
        ;

    Prototype::new(Some(base_mob))
        (Desc::new("wraith", 74, HOTPINK))
        (Stats::new(8, &[Hands]))
        (Spawn::new(Category::Mob).biome(Dungeon).depth(6))
        ;

    Prototype::new(Some(base_mob))
        (Desc::new("octopus", 63, DARKTURQUOISE))
        (Stats::new(10, &[]))
        (Spawn::new(Category::Mob).depth(7))
        ;

    Prototype::new(Some(base_mob))
        (Desc::new("efreet", 78, ORANGE))
        (Stats::new(12, &[]))
        (Spawn::new(Category::Mob).depth(8))
        ;

    Prototype::new(Some(base_mob))
        (Desc::new("serpent", 94, CORAL))
        (Stats::new(15, &[]))
        (Spawn::new(Category::Mob).biome(Dungeon).depth(9))
        ;

    // Items
    Prototype::new(None)
        (Desc::new("heart", 89, RED))
        (Spawn::new(Category::Consumable).commonness(50))
        (Item { item_type: ItemType::Instant, ability: Ability::HealInstant(2) })
        ;

    Prototype::new(None)
        (Desc::new("sword", 84, GAINSBORO))
        (Spawn::new(Category::Equipment).commonness(100))
        (Stats::new(0, &[]).attack(5).mana(-3))
        (Item { item_type: ItemType::MeleeWeapon, ability: Ability::Multi(vec![]) })
        ;

    Prototype::new(None)
        (Desc::new("throwing knives", 90, GAINSBORO))
        (Spawn::new(Category::Equipment).commonness(500))
        (Stats::new(0, &[]).ranged_range(5).ranged_power(5))
        (Item { item_type: ItemType::RangedWeapon, ability: Ability::Multi(vec![]) })
        ;

    Prototype::new(None)
        (Desc::new("helmet", 85, GAINSBORO))
        (Spawn::new(Category::Equipment).commonness(100))
        (Stats::new(0, &[]).protection(2).mana(-1))
        (Item { item_type: ItemType::Helmet, ability: Ability::Multi(vec![]) })
        ;

    Prototype::new(None)
        (Desc::new("armor", 91, GAINSBORO))
        (Spawn::new(Category::Equipment).commonness(100))
        (Stats::new(0, &[]).protection(5).mana(-3))
        (Item { item_type: ItemType::Armor, ability: Ability::Multi(vec![]) })
        ;
}
