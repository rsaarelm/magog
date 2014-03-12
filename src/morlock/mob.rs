use color::rgb::RGB;
use color::rgb::consts::*;

use calx::app::{SPRITE_INDEX_START};

use area::Location;

pub enum MobType {
    Player,
    Morlock,
    BigMorlock,
    Centipede,
    TimeEater,
}

pub struct MobData {
    sprite: uint,
    max_hits: uint,
    color: RGB<u8>,
    name: ~str,
}

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
            Player =>       MobData { sprite: SPRITE_INDEX_START + 51, max_hits: 5, color: AZURE, name: ~"you" },
            Morlock =>      MobData { sprite: SPRITE_INDEX_START + 59, max_hits: 1, color: LIGHTSLATEGRAY, name: ~"morlock" },
            BigMorlock =>   MobData { sprite: SPRITE_INDEX_START + 60, max_hits: 3, color: GOLD, name: ~"big morlock" },
            Centipede =>    MobData { sprite: SPRITE_INDEX_START + 61, max_hits: 2, color: DARKCYAN, name: ~"centipede" },
            TimeEater =>    MobData { sprite: SPRITE_INDEX_START + 62, max_hits: 6, color: CRIMSON, name: ~"time eater" },
        }
    }

    pub fn data(&self) -> MobData { Mob::type_data(self.t) }
}
