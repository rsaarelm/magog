use std::rand;
use std::rand::Rng;
use cgmath::vector::{Vector2};
use world::system::{World, Entity, EngineLogic};
use world::spatial::{Location, Position, DIRECTIONS6};
use world::area::Area;

#[deriving(Clone, Show)]
pub struct MobComp {
    pub t: MobType,
}

impl MobComp {
    pub fn new(t: MobType) -> MobComp {
        MobComp {
            t: t,
        }
    }

}

pub mod quirk {
#[deriving(Eq, PartialEq, Clone)]
pub enum Quirk {
    /// Moves 1/3 slower than usual
    Slow,
    /// Moves 1/3 faster than usual, stacks with Quick
    Fast,
    /// Moves 1/3 faster than usual, stacks with Fast
    Quick,
}
}

/// Trait for entities that are mobile things.
pub trait Mob {
    fn acts_this_frame(&self) -> bool;
    fn has_quirk(&self, q: quirk::Quirk) -> bool;
    fn mob_type(&self) -> MobType;
    fn update_ai(&mut self);

    /// Try to move the mob in a direction, then try to roll around obstacles
    /// if the direction is blocked.
    fn smart_move(&mut self, dir8: uint) -> Option<Vector2<int>>;

}

impl Mob for Entity {
    fn acts_this_frame(&self) -> bool {
        if !self.has::<MobComp>() { return false; }

        // Go through a cycle of 5 phases to get 4 possible speeds.
        // System idea from Jeff Lait.
        let phase = self.world().get_tick() % 5;
        match phase {
            0 => return true,
            1 => return self.has_quirk(quirk::Fast),
            2 => return true,
            3 => return self.has_quirk(quirk::Quick),
            4 => return !self.has_quirk(quirk::Slow),
            _ => fail!("Invalid action phase"),
        }
    }

    fn has_quirk(&self, q: quirk::Quirk) -> bool {
        if q == quirk::Fast && self.mob_type() == GridBug { return true; }

        false
    }

    fn mob_type(&self) -> MobType {
        self.into::<MobComp>().unwrap().t
    }

    fn update_ai(&mut self) {
        if self.mob_type() == GridBug {
            // Grid bugs move only non-diagonally. Even though horizontal
            // non-diagonal movement actually involves teleporting through
            // walls...
            let delta = *rand::task_rng()
                .choose(&[
                        Vector2::new(-1, -1),
                        Vector2::new( 1, -1),
                        Vector2::new( 1,  1),
                        Vector2::new(-1,  1),
                        ])
                .unwrap();
            self.move(&delta);
            return;
        }

        self.move(rand::task_rng().choose(DIRECTIONS6.as_slice()).unwrap());
    }

    fn smart_move(&mut self, dir8: uint) -> Option<Vector2<int>> {
        static SMART_MOVE: &'static [&'static [Vector2<int>]] = &[
            &[DIRECTIONS6[0], DIRECTIONS6[5], DIRECTIONS6[1]],
            &[DIRECTIONS6[1], DIRECTIONS6[0], DIRECTIONS6[2]],
            &[DIRECTIONS6[2], DIRECTIONS6[1], DIRECTIONS6[3]],
            &[DIRECTIONS6[3], DIRECTIONS6[2], DIRECTIONS6[4]],
            &[DIRECTIONS6[4], DIRECTIONS6[3], DIRECTIONS6[5]],
            &[DIRECTIONS6[5], DIRECTIONS6[4], DIRECTIONS6[0]],

            // Right sideways move zigzag.
            &[DIRECTIONS6[1], DIRECTIONS6[2]],
            &[DIRECTIONS6[2], DIRECTIONS6[1]],

            // Left sideways move zigzag.
            &[DIRECTIONS6[5], DIRECTIONS6[4]],
            &[DIRECTIONS6[4], DIRECTIONS6[5]],
            ];
        // "horizontal" movement is a zig-zag since there's no natural hex axis
        // in that direction. Find out the grid column the mob is on and
        // determine whether to zig or zag based on that.
        let loc = self.location();
        let zag = ((loc.x - loc.y) % 2) as uint;

        let deltas = SMART_MOVE[match dir8 {
                0 => 0,
                1 => 1,
                2 => 6 + zag,
                3 => 2,
                4 => 3,
                5 => 4,
                6 => 8 + zag,
                7 => 5,
                _ => fail!("Invalid dir8"),
            }];

        for delta in deltas.iter() {
            if self.move(delta) { return Some(*delta); }
        }

        None
    }
}

#[deriving(Eq, PartialEq, Clone, Show)]
pub enum MobType {
    Player,
    Dreg,
    GridBug,
    Serpent,
}

pub trait Mobs {
    fn mobs_at(&self, loc: Location) -> Vec<Entity>;
    fn mobs(&self) -> Vec<Entity>;
    fn player(&self) -> Option<Entity>;
    fn player_has_turn(&self) -> bool;
    fn clear_npcs(&mut self);
    fn update_mobs(&mut self);
}

impl Mobs for World {
    fn mobs_at(&self, loc: Location) -> Vec<Entity> {
        self.entities_at(loc).iter().filter(|e| e.has::<MobComp>())
            .map(|e| e.clone()).collect()
    }

    fn mobs(&self) -> Vec<Entity> {
        self.entities().filter(|e| e.has::<MobComp>())
            .map(|e| e.clone()).collect()
    }

    fn player(&self) -> Option<Entity> {
        for e in self.mobs().iter() {
            if e.mob_type() == Player {
                return Some(e.clone());
            }
        }
        None
    }

    fn player_has_turn(&self) -> bool {
        match self.player() {
            Some(p) => p.acts_this_frame(),
            _ => false
        }
    }

    fn clear_npcs(&mut self) {
        for e in self.mobs().mut_iter() {
            if e.mob_type() != Player {
                e.delete();
            }
        }
    }

    fn update_mobs(&mut self) {
        for mob in self.mobs().mut_iter() {
            if !mob.acts_this_frame() { continue; }
            if mob.mob_type() == Player { continue; }
            mob.update_ai();
        }

        self.advance_frame();
    }
}
