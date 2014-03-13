use color::rgb::consts::*;

use area::Location;
use transform::Transform;
use sprite::{Sprite, tile};
use sprite;

#[deriving(Eq, Clone)]
pub enum MobType {
    Player,
    Morlock,
    BigMorlock,
    Centipede,
    TimeEater,
}

pub struct MobData {
    max_hits: uint,
    name: ~str,
}

#[deriving(Clone)]
pub struct Mob {
    t: MobType,
    loc: Location,
    hits: uint,
    moved: bool,
}

impl Mob {
    pub fn new(t: MobType, loc: Location) -> Mob {
       Mob {
           t: t,
           loc: loc,
           hits: Mob::type_data(t).max_hits,
           moved: false,
       }
    }

    // XXX: Initializing the data struct for every return. Quite inefficient
    // compared to having a bunch of static values and returning references to
    // those, but doing that would have involved either extra indexing
    // boilerplate or using macros.
    pub fn type_data(t: MobType) -> MobData {
        match t {
            Player =>       MobData { max_hits: 5, name: ~"you" },
            Morlock =>      MobData { max_hits: 1, name: ~"morlock" },
            BigMorlock =>   MobData { max_hits: 3, name: ~"big morlock" },
            Centipede =>    MobData { max_hits: 2, name: ~"centipede" },
            TimeEater =>    MobData { max_hits: 6, name: ~"time eater" },
        }
    }

    pub fn data(&self) -> MobData { Mob::type_data(self.t) }

    pub fn sprites(&self, xf: &Transform) -> ~[Sprite] {
        let mut ret : ~[Sprite] = ~[];

        match self.t {
            Player => {
                ret.push(Sprite::new(tile(51), xf.to_screen(self.loc), sprite::BLOCK_Z, AZURE));
            },
            Morlock => {
                ret.push(Sprite::new(tile(59), xf.to_screen(self.loc), sprite::BLOCK_Z, LIGHTSLATEGRAY));
            },
            BigMorlock => {
                ret.push(Sprite::new(tile(60), xf.to_screen(self.loc), sprite::BLOCK_Z, GOLD));
            },
            Centipede => {
                ret.push(Sprite::new(tile(61), xf.to_screen(self.loc), sprite::BLOCK_Z, DARKCYAN));
            },
            TimeEater => {
                ret.push(Sprite::new(tile(62), xf.to_screen(self.loc), sprite::BLOCK_Z, CRIMSON));
            },
        };
        ret
    }
}
