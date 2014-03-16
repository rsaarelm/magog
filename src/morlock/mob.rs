use time;

use color::rgb::consts::*;
use cgmath::vector::{Vec2};
use cgmath::point::{Point};

use area::Location;
use transform::Transform;
use sprite::{Sprite, tile};
use sprite;
use misc::{cycle_anim, single_anim};

// How many seconds to show the hurt blink.
static HURT_TIME_S : f64 = 0.2;
static DEATH_TIME_S : f64 = 0.6;

#[deriving(Eq, Clone)]
pub enum AnimState {
    Asleep,
    Awake,
    Hurt(f64),
    Dying(f64),
    Dead,
}

#[deriving(Eq, Clone)]
pub enum MobType {
    Player,
    Morlock,
    Centipede,
    BigMorlock,
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
    hits: int,
    anim_state: AnimState,
    // Player only.
    ammo: uint,
}

impl Mob {
    pub fn new(t: MobType, loc: Location) -> Mob {
       Mob {
           t: t,
           loc: loc,
           hits: Mob::type_data(t).max_hits as int,
           anim_state: Awake,
           ammo: 6,
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
            Centipede =>    MobData { max_hits: 2, name: ~"centipede" },
            BigMorlock =>   MobData { max_hits: 3, name: ~"big morlock" },
            TimeEater =>    MobData { max_hits: 4, name: ~"time eater" },
        }
    }

    pub fn data(&self) -> MobData { Mob::type_data(self.t) }

    pub fn is_alive(&self) -> bool { self.hits > 0 }

    pub fn update_anim(&mut self) {
        match self.anim_state {
            Hurt(start_t) => {
                let t = time::precise_time_s();
                if t - start_t > HURT_TIME_S {
                    self.anim_state = Awake;
                }
            }
            Dying(start_t) => {
                let t = time::precise_time_s();
                if t - start_t > DEATH_TIME_S {
                    self.anim_state = Dead;
                }
            }
            _ => (),
        }
    }

    pub fn sprites(&self, xf: &Transform) -> ~[Sprite] {
        let mut ret : ~[Sprite] = ~[];
        let pos = xf.to_screen(self.loc);

        let bob = Vec2::new(0.0f32, *cycle_anim(0.25f64, &[0.0f32, -1.0f32]));

        match self.t {
            Player => {
                ret.push(Sprite::new(tile(51), pos, sprite::BLOCK_Z, AZURE));
            },
            Morlock => {
                ret.push(Sprite::new(tile(59), pos, sprite::BLOCK_Z, LIGHTSLATEGRAY));
            },
            Centipede => {
                ret.push(Sprite::new(tile(61), pos, sprite::BLOCK_Z, DARKCYAN));
            },
            BigMorlock => {
                ret.push(Sprite::new(tile(60), pos, sprite::BLOCK_Z, GOLD));
            },
            TimeEater => {
                ret.push(Sprite::new(tile(62), pos, sprite::BLOCK_Z, CRIMSON));
            },
        };


        match self.anim_state {
            Awake => {
                if self.t != Player {
                    for s in ret.mut_iter() {
                        s.pos = s.pos.add_v(&bob);
                    }
                }
            },
            Hurt(_) => {
                for s in ret.mut_iter() {
                    s.color = *cycle_anim(0.05f64, &[BLACK, WHITE]);
                }
            }
            Dying(start_time) => {
                ret = ~[Sprite::new(
                    *single_anim(start_time, 0.1f64, &[tile(64), tile(65), tile(66), tile(67), tile(68)]),
                    pos, sprite::BLOCK_Z, MAROON)];
            }
            Dead => {
                ret = ~[Sprite::new(tile(68), pos, sprite::FLOOR_Z, MAROON)];
            }
            _ => (),
        }
        ret
    }
}
