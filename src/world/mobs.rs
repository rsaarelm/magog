use std::rand;
use std::rand::Rng;
use cgmath::vector::{Vector2};
use world::world::{World, Location, DIRECTIONS6};
use world::area::Area;

pub struct Mob {
    pub id: MobId,
    pub loc: Location,
    pub t: MobType,
}

impl Mob {
    pub fn new(t: MobType) -> Mob {
        Mob {
            id: MobId(0),
            loc: Location::new(0, 0),
            t: t,
        }
    }

    pub fn has_quirk(&self, q: quirk::Quirk) -> bool {
        if q == quirk::Fast && self.t == GridBug { return true; }

        false
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

#[deriving(Eq, PartialEq, Hash)]
pub struct MobId(pub u64);

impl MobId {
    fn map<T>(&self, f: |&Mob| -> T) -> T {
        World::map(|w| f(w.mob(*self)))
    }

    fn map_mut<T>(&self, f: |&mut Mob| -> T) -> T {
        World::map_mut(|w| f(w.mob_mut(*self)))
    }

    pub fn loc(&self) -> Location { self.map(|m| m.loc) }
    pub fn typ(&self) -> MobType { self.map(|m| m.t) }

    pub fn move(&self, delta: &Vector2<int>) -> bool {
        let new_loc = self.loc() + *delta;

        if World::map(|w| w.is_walkable(new_loc)) {
            self.map_mut(|m| m.loc = new_loc);
            return true;
        }
        return false;
    }

    /// Try to move the mob in a direction, then try to roll around obstacles
    /// if the direction is blocked.
    pub fn smart_move(&self, dir8: uint) -> Option<Vector2<int>> {
        fn moves(ctx: &MobId, deltas: &[Vector2<int>]) ->
            Option<Vector2<int>> {
            for delta in deltas.iter() {
                if ctx.move(delta) { return Some(*delta); }
            }

            None
        }

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
        let loc = self.loc();
        let zag = ((loc.x - loc.y) % 2) as uint;

        moves(self, SMART_MOVE[
        match dir8 {
            0 => 0,
            1 => 1,
            2 => 6 + zag,
            3 => 2,
            4 => 3,
            5 => 4,
            6 => 8 + zag,
            7 => 5,
            _ => fail!("Invalid dir8"),
        }])
    }

    pub fn exists(&self) -> bool { World::map(|w| w.mob_exists(*self)) }

    pub fn acts_this_frame(&self) -> bool {
        if !self.exists() { return false; }

        // Go through a cycle of 5 phases to get 4 possible speeds.
        // System idea from Jeff Lait.
        let phase = World::map(|w| w.get_tick() % 5);
        match phase {
            0 => return true,
            1 => return self.map(|m| m.has_quirk(quirk::Fast)),
            2 => return true,
            3 => return self.map(|m| m.has_quirk(quirk::Quick)),
            4 => return !self.map(|m| m.has_quirk(quirk::Slow)),
            _ => fail!("Invalid action phase"),
        }
    }

    pub fn update_ai(&self) {
        if !self.acts_this_frame() { return; }
        if self.typ() == Player { return; }

        if self.typ() == GridBug {
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
        }
    }
}

#[deriving(Eq, PartialEq, Clone)]
pub enum MobType {
    Player,
    Dreg,
    GridBug,
    Serpent,
}

pub trait Mobs {
    fn mobs_at(&self, loc: Location) -> Vec<MobId>;

    fn has_mobs_at(&self, loc: Location) -> bool {
        self.mobs_at(loc).len() > 0
    }

    /// Returns mob reference without checking if it exists.
    fn mob<'a>(&'a self, id: MobId) -> &'a Mob;
    fn mob_mut<'a>(&'a mut self, id: MobId) -> &'a mut Mob;
    fn mob_exists(&self, id: MobId) -> bool;
    fn player(&self) -> Option<MobId>;
    fn clear_npcs(&mut self);
}

impl Mobs for World {
    fn mobs_at(&self, loc: Location) -> Vec<MobId> {
        let mut ret = vec!();
        for (&id, mob) in self.mobs.iter() {
            if mob.loc == loc {
                ret.push(id);
            }
        }
        ret
    }

    fn mob<'a>(&'a self, id: MobId) -> &'a Mob {
        self.mobs.find(&id).unwrap()
    }


    fn mob_mut<'a>(&'a mut self, id: MobId) -> &'a mut Mob {
        self.mobs.find_mut(&id).unwrap()
    }

    fn mob_exists(&self, id: MobId) -> bool { self.mobs.find(&id).is_some() }

    fn player(&self) -> Option<MobId> {
        for (_, mob) in self.mobs.iter() {
            if mob.t == Player {
                return Some(mob.id);
            }
        }
        None
    }

    fn clear_npcs(&mut self) {
        let npcs: Vec<MobId> = self.mobs.iter()
            .filter(|&(_, m)| m.t != Player)
            .map(|(&i, _)| i).collect();
        for &id in npcs.iter() {
            self.remove_mob(id);
        }
    }
}
