use time;
use color::rgb::consts::*;
use color::rgb::{RGB};
use cgmath::point::{Point2};
use cgmath::vector::{Vector2};
use cgmath::point::{Point};
use timing::{cycle_anim, single_anim};
use world::area::{Location};
use world::sprite::{BLOCK_Z, DrawContext};

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
    Invisible,
}

#[deriving(Eq, Clone)]
pub enum MobType {
    Player,
    Morlock,
    Centipede,
    BigMorlock,
    TimeEater,
    Serpent,
}

pub struct MobData {
    pub max_hits: uint,
    pub name: ~str,
}

#[deriving(Clone)]
pub struct Mob {
    pub t: MobType,
    pub loc: Location,
    pub hits: int,
    pub anim_state: AnimState,
    // Player only.
    pub ammo: uint,
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
            Player =>       MobData { max_hits: 5, name: "you".to_owned() },
            Morlock =>      MobData { max_hits: 1, name: "morlock".to_owned() },
            Centipede =>    MobData { max_hits: 2, name: "centipede".to_owned() },
            BigMorlock =>   MobData { max_hits: 3, name: "big morlock".to_owned() },
            TimeEater =>    MobData { max_hits: 4, name: "time eater".to_owned() },
            Serpent =>      MobData { max_hits: 1, name: "serpent".to_owned() },
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

    /// Should the mob be shown doing an idle alert animation?
    fn is_bobbing(&self) -> bool {
        self.anim_state == Awake && self.t != Player
    }

    fn color(&self) -> RGB<u8> {
        match self.t {
            Player => AZURE,
            Morlock => LIGHTSLATEGRAY,
            Centipede => DARKCYAN,
            BigMorlock => GOLD,
            TimeEater => CRIMSON,
            Serpent => CORAL,
        }
    }

    pub fn draw<C: DrawContext>(&self, ctx: &mut C, pos: &Point2<f32>) {
        // Handle special animations.
        match self.anim_state {
            Dying(start_time) => {
                let frame = *single_anim(
                    start_time, 0.1f64, &[64, 65, 66, 67, 68]);
                ctx.draw(frame, pos, BLOCK_Z, &MAROON);
                return;
            }
            Dead => {
                ctx.draw(68, pos, BLOCK_Z, &MAROON);
                return;
            }
            Invisible => {
                return;
            }
            _ => ()
        };

        let body_pos = if self.is_bobbing() {
            pos.add_v(&Vector2::new(0.0f32, *cycle_anim(0.25f64, &[0.0f32, 1.0f32])))
        } else {
            *pos
        };

        let color = match self.anim_state {
            Hurt(_) => *cycle_anim(0.05f64, &[BLACK, WHITE]),
            _ => self.color()
        };

        match self.t {
            Player => {
                ctx.draw(51, &body_pos, BLOCK_Z, &color);
            },
            Morlock => {
                ctx.draw(59, &body_pos, BLOCK_Z, &color);
            },
            Centipede => {
                ctx.draw(61, &body_pos, BLOCK_Z, &color);
            },
            BigMorlock => {
                ctx.draw(60, &body_pos, BLOCK_Z, &color);
            },
            TimeEater => {
                ctx.draw(62, &body_pos, BLOCK_Z, &color);
            },
            Serpent => {
                // Body
                ctx.draw(94, &body_pos, BLOCK_Z, &color);
                // Ground mound
                ctx.draw(95, pos, BLOCK_Z, &color);
            }
        };
    }
}
