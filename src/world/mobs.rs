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
            id: 0,
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
#[deriving(Eq, Clone)]
pub enum Quirk {
    /// Moves 1/3 slower than usual
    Slow,
    /// Moves 1/3 faster than usual, stacks with Quick
    Fast,
    /// Moves 1/3 faster than usual, stacks with Fast
    Quick,
}
}

pub type MobId = u64;

#[deriving(Eq, Clone)]
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

    fn mut_mob<'a>(&'a mut self, id: MobId) -> &'a mut Mob;

    fn mob_exists(&self, id: MobId) -> bool;

    fn player(&self) -> Option<MobId>;

    fn move(&mut self, id: MobId, delta: &Vector2<int>) -> bool;

    /// Try to move the mob in a direction, then try to roll around obstacles
    /// if the direction is blocked.
    fn smart_move(&mut self, id: MobId, dir8: uint) -> Option<Vector2<int>>;
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


    fn mut_mob<'a>(&'a mut self, id: MobId) -> &'a mut Mob {
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

    fn move(&mut self, id: MobId, delta: &Vector2<int>) -> bool {
        let new_loc = self.mob(id).loc + *delta;

        if self.is_walkable(new_loc) {
            self.mut_mob(id).loc = new_loc;
            return true;
        }
        return false;
    }

    fn smart_move(&mut self, id: MobId, dir8: uint) -> Option<Vector2<int>> {
        fn moves(ctx: &mut World, id: MobId, deltas: &[Vector2<int>]) ->
            Option<Vector2<int>> {
            for delta in deltas.iter() {
                if ctx.move(id, delta) { return Some(*delta); }
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
        let loc = self.mob(id).loc;
        let zag = ((loc.x - loc.y) % 2) as uint;

        moves(self, id, SMART_MOVE[
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
}
