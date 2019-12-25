use calx::{Clamp, Deciban};

mod ai;

mod animations;
pub use animations::{Anim, AnimState, LerpLocation, PhysicsSpace, PhysicsVector};

mod command;
pub use command::{ActionOutcome, Command};

mod components;
pub use components::Icon;

mod effect;
pub use effect::Ability;

mod extract;
pub use extract::ExternalEntity;

mod flags;

mod fov;

mod grammar;

mod item;
pub use item::{ItemType, Slot};

mod location;
pub use location::{Location, Portal};

mod location_set;

mod mapsave;
pub use mapsave::MapSave;

mod map;

mod msg;
pub use msg::{register_msg_receiver, MsgReceiver};

mod mutate;

mod query;

mod sector;
pub use sector::{Sector, SectorDir, SectorVec, WorldSkeleton, SECTOR_HEIGHT, SECTOR_WIDTH};

mod spatial;
mod spec;
mod stats;

pub mod terrain;
pub use terrain::Terrain;

mod vaults;

mod volume;

mod world;
pub use crate::world::{Ecs, World, WorldSeed};

mod world_cache;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FovStatus {
    Seen,
    Remembered,
}

pub type Rng = rand_xorshift::XorShiftRng;

/// Object-safe version of `rand::distributions::Distribution`.
pub trait Distribution<T> {
    fn sample(&self, rng: &mut Rng) -> T;
}

/// The combat formula.
///
/// Given a deciban roll and the relevant stats, determine amount of damage dealt.
/// Advantage is attacker skill - target defense.
pub fn attack_damage(roll: f32, advantage: i32, weapon_power: i32) -> i32 {
    const MAX_DAMAGE_MULTIPLIER: f32 = 4.0;

    let roll = roll + advantage as f32;
    (weapon_power as f32 * (0.0..=MAX_DAMAGE_MULTIPLIER).clamp((roll - 2.0) * 0.05)) as i32
}

/// Standard deciban roll, clamp into [-20, 20].
pub fn roll(rng: &mut impl rand::Rng) -> f32 { (-20.0..=20.0).clamp(rng.gen::<Deciban>().0) }
