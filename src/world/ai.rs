use std::rand;
use std::rand::Rng;
use cgmath::vector::{Vector2};
use world::world::World;
use world::mobs::*;

pub trait AI {
    fn player_has_turn(&self) -> bool;
    fn acts_this_frame(&self, id: MobId) -> bool;
    fn update_mobs(&mut self);
}

impl AI for World {
    fn player_has_turn(&self) -> bool {
        match self.player() {
            Some(p) => self.acts_this_frame(p),
            _ => false
        }
    }

    fn acts_this_frame(&self, id: MobId) -> bool {
        if !self.mob_exists(id) { return false; }

        let mob = self.mob(id);

        // Go through a cycle of 5 phases to get 4 possible speeds.
        // System idea from Jeff Lait.
        let phase = self.get_tick() % 5;
        match phase {
            0 => return true,
            1 => return mob.has_quirk(quirk::Fast),
            2 => return true,
            3 => return mob.has_quirk(quirk::Quick),
            4 => return !mob.has_quirk(quirk::Slow),
            _ => fail!("Invalid action phase"),
        }
    }

    fn update_mobs(&mut self) {
        // XXX: Horribly inefficent borrow checker avoidance dance.
        let buf : Vec<(MobId, Mob)> = self.mobs.iter().map(|(&x, &y)| (x, y)).collect();
        for &(id, mob) in buf.iter() {
            if mob.t == Player {
                continue;
            }

            if mob.t == GridBug {
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
                self.move(id, &delta);
            }
        }

        self.advance_frame();
    }
}
