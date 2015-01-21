use std::default::Default;
use util::color;
use ecs::{Component};
use entity::{Entity};
use components::{Spawn, Category};
use components::{Desc, MapMemory, Health};
use components::{Brain, BrainState, Alignment};
use components::{Item};
use item::{ItemType};
use stats::{Stats, Intrinsic};
use ability::Ability;
use {Biome};
use world;

#[derive(Copy)]
pub struct Prototype {
    pub target: Entity
}

impl Prototype {
    pub fn new(parent: Option<Entity>) -> Prototype {
        Prototype {
            target: world::with_mut(|w| w.ecs.new_entity(parent))
        }
    }
}

impl<C: Component> Fn(C,) -> Prototype for Prototype {
    extern "rust-call" fn call(&self, (comp,): (C,)) -> Prototype {
        comp.add_to(self.target);
        *self
    }
}

/// Only call at world init!
pub fn init() {

    /*
//  Symbol   power, depth, biome, sprite, color,        intrinsics
    Player:     6,  -1, Anywhere, 51, &AZURE,            f!(Hands);
    Dreg:       1,   1, Anywhere, 72, &OLIVE,            f!(Hands);
    Snake:      1,   1, Overland, 71, &GREEN,            f!();
    Ooze:       3,   3, Dungeon,  77, &LIGHTSEAGREEN,    f!();
    Flayer:     4,   4, Anywhere, 75, &INDIANRED,        f!();
    Ogre:       6,   5, Anywhere, 73, &DARKSLATEGRAY,    f!(Hands);
    Wraith:     8,   6, Dungeon,  74, &HOTPINK,          f!(Hands);
    Octopus:    10,  7, Anywhere, 63, &DARKTURQUOISE,    f!();
    Efreet:     12,  8, Anywhere, 78, &ORANGE,           f!();
    Serpent:    15,  9, Dungeon,  94, &CORAL,            f!();
    */

    let base_mob = Prototype::new(None)
        (Brain { state: BrainState::Asleep, alignment: Alignment::Evil })
        ({let h: Health = Default::default(); h})
        .target;

    // Init the prototypes

    // Player
    Prototype::new(Some(base_mob))
        (Brain { state: BrainState::PlayerControl, alignment: Alignment::Good })
        (Desc::new("player", 51, color::AZURE))
        (Stats { power: 6, intrinsics: Intrinsic::Hands as u32 })
        (MapMemory::new())
        ;

    // Enemies
    Prototype::new(Some(base_mob))
        (Desc::new("dreg", 72, color::OLIVE))
        (Stats { power: 1, intrinsics: Intrinsic::Hands as u32 })
        (Spawn { biome: Biome::Anywhere, commonness: 1000, min_depth: 1, category: Category::Mob })
        ;

    Prototype::new(Some(base_mob))
        (Desc::new("snake", 71, color::GREEN))
        (Stats { power: 1, intrinsics: 0 })
        (Spawn { biome: Biome::Overland, commonness: 1000, min_depth: 1, category: Category::Mob })
        ;

    Prototype::new(Some(base_mob))
        (Desc::new("ooze", 77, color::LIGHTSEAGREEN))
        (Stats { power: 3, intrinsics: 0 })
        (Spawn { biome: Biome::Dungeon, commonness: 1000, min_depth: 3, category: Category::Mob })
        ;
    // TODO: More mob types

    // Items
    Prototype::new(None)
        (Desc::new("heart", 89, color::RED))
        (Spawn { biome: Biome::Anywhere, commonness: 1000, min_depth: 1, category: Category::Consumable })
        (Item { item_type: ItemType::Instant, ability: Ability::HealInstant(2) })
        ;

    Prototype::new(None)
        (Desc::new("sword", 84, color::GAINSBORO))
        (Spawn { biome: Biome::Anywhere, commonness: 500, min_depth: 1, category: Category::Equipment })
        (Item { item_type: ItemType::MeleeWeapon, ability: Ability::Multi(vec![]) })
        (Stats { power: 5, intrinsics: 0 })
        ;
}
