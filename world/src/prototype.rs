use std::default::Default;
use util::color::*;
use ecs::{Component, ComponentAccess};
use entity::{Entity};
use components::{Spawn, Category, IsPrototype};
use components::{Desc, MapMemory, Health};
use components::{Brain, BrainState, Alignment, Colonist};
use stats::{Stats};
use stats::Intrinsic::*;
use Biome::*;
use world;

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
    type Output = Prototype;
    extern "rust-call" fn call(&self, (comp,): (C,)) -> Prototype {
        comp.add_to(self.target);
        *self
    }
}

/// Only call at world init!
pub fn init() {
    let base_mob = Prototype::new(None)
        (Brain { state: BrainState::Asleep, alignment: Alignment::Indigenous })
        ({let h: Health = Default::default(); h})
        .target;

    let colonist = Prototype::new(None)
        (Brain { state: BrainState::Asleep, alignment: Alignment::Colonist })
        ({let h: Health = Default::default(); h})
        .target;

    // Init the prototypes

    // Player
    Prototype::new(Some(base_mob))
        (Brain { state: BrainState::PlayerControl, alignment: Alignment::Phage })
        (Desc::new("phage", 40, CYAN))
        (Stats::new(2, &[Fast]).attack(3))
        (MapMemory::new())
        ;

    // Enemies

    // Indigenous
    Prototype::new(Some(base_mob))
        (Desc::new("hopper", 32, YELLOW))
        (Stats::new(4, &[]).protection(-2))
        (Spawn::new(Category::Mob).commonness(2000))
        ;

    Prototype::new(Some(base_mob))
        (Desc::new("stalker", 60, ORCHID))
        (Stats::new(4, &[]))
        (Spawn::new(Category::Mob))
        ;

    Prototype::new(Some(base_mob))
        (Desc::new("metawasp", 58, ORANGERED))
        // Glass cannon
        (Stats::new(4, &[Fast]).protection(-1).attack(2))
        (Spawn::new(Category::Mob).commonness(600))
        ;

    // Can open doors, good for base attack.
    Prototype::new(Some(base_mob))
        (Desc::new("space monkey", 46, LAWNGREEN))
        (Stats::new(6, &[Hands]))
        (Spawn::new(Category::Mob).commonness(600))
        ;

    Prototype::new(Some(base_mob))
        (Desc::new("rumbler", 38, OLIVE))
        (Stats::new(8, &[Slow]))
        (Spawn::new(Category::Mob).commonness(100))
        ;

    // Colonist enemies

    Prototype::new(Some(colonist))
        (Desc::new("colonist", 34, DARKORANGE))
        (Stats::new(6, &[Hands]))
        (Spawn::new(Category::Mob).biome(Base))
        (Colonist::new())
        ;

    // TODO: Ranged attack
    Prototype::new(Some(colonist))
        (Desc::new("marine", 36, DARKOLIVEGREEN))
        (Stats::new(8, &[Hands]))
        (Spawn::new(Category::Mob).biome(Base).commonness(400))
        (Colonist::new())
        ;

    // TODO: Ranged attack
    Prototype::new(Some(colonist))
        (Desc::new("cyber controller", 42, LIGHTSLATEGRAY))
        (Stats::new(12, &[Slow, Hands, Robotic]))
        (Colonist::new())
        (Spawn::new(Category::Mob).biome(Base).commonness(40))
        ;

    // Dogs count as colonists because of terran DNA
    Prototype::new(Some(colonist))
        (Desc::new("dog", 44, OLIVE))
        (Stats::new(4, &[]))
        (Spawn::new(Category::Mob).biome(Base))
        (Colonist::new())
        ;

    // Robots don't count as colonists, being completely inorganic
    Prototype::new(Some(colonist))
        (Desc::new("robot", 62, SILVER))
        (Stats::new(6, &[Hands, Robotic, Slow]))
        (Spawn::new(Category::Mob).biome(Base).commonness(200))
        ;
}
