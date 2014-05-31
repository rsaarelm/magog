use world::world::{World, Location};

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

    /// Returns mob reference without checking if it exists.
    fn mob<'a>(&'a self, id: MobId) -> &'a Mob;

    fn mob_exists(&self, id: MobId) -> bool;

    fn player<'a>(&'a mut self) -> &'a mut Mob;
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

    fn mob_exists(&self, id: MobId) -> bool { self.mobs.find(&id).is_some() }

    fn player<'a>(&'a mut self) -> &'a mut Mob {
        for (_, mob) in self.mobs.mut_iter() {
            if mob.t == Player {
                return mob;
            }
        }
        fail!("Player doesn't exit");
    }
}
