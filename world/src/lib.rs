extern crate rand;
extern crate ron;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate euclid;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate error_chain;
extern crate calx_alg;
extern crate calx_grid;
#[macro_use]
extern crate calx_ecs;

use calx_alg::Deciban;

/// Helper macro for formatting textual event messages.
macro_rules! msg {
    ($ctx: expr, $fmt:expr) => {
        let __event = Event::Msg($ctx.expand_template($fmt));
        $ctx.push_event(__event);
    };

    ($ctx: expr, $fmt:expr, $($arg:expr),*) => {
        let __event = Event::Msg($ctx.expand_template(&format!($fmt, $($arg),*)));
        $ctx.push_event(__event);
    };
}

mod command;
pub use command::{Command, CommandResult};

mod components;
pub use components::Icon;

mod effect;

mod event;
pub use event::Event;

mod flags;

mod form;
pub use form::{Form, FORMS};

mod fov;

mod grammar;

mod item;
pub use item::{Slot, ItemType};

mod location;
pub use location::{Location, Portal, Sector};

mod location_set;

mod mapfile;
pub use mapfile::{save_prefab, load_prefab};

pub mod mapgen;

mod mutate;
pub use mutate::Mutate;

mod query;
pub use query::Query;

mod spatial;
mod stats;

mod terraform;
pub use terraform::{Terraform, TerrainQuery};

pub mod terrain;
pub use terrain::Terrain;

mod volume;

mod world;
pub use world::{Ecs, World};

mod worldgen;

/// Standard Prefab type, terrain type and spawn name list.
pub type Prefab = calx_grid::Prefab<(Terrain, Vec<String>)>;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FovStatus {
    Seen,
    Remembered,
}

pub type Rng = calx_alg::EncodeRng<rand::XorShiftRng>;

/// The combat formula.
///
/// Given a deciban roll and the relevant stats, determine amount of damage dealt.
/// Advantage is attacker skill - target defense.
pub fn attack_damage(roll: f32, advantage: i32, weapon_power: i32) -> i32 {
    const MAX_DAMAGE_MULTIPLIER: f32 = 4.0;

    let roll = roll + advantage as f32;
    (weapon_power as f32 * calx_alg::clamp(0.0, MAX_DAMAGE_MULTIPLIER, (roll - 2.0) * 0.05)) as i32
}

/// Standard deciban roll, clamp into [-20, 20].
pub fn roll<R: rand::Rng>(rng: &mut R) -> f32 {
    calx_alg::clamp(-20.0, 20.0, rng.gen::<Deciban>().0)
}

pub mod errors {
    error_chain! {
        foreign_links {
            Io(::std::io::Error);
            RonSerialize(::ron::ser::Error);
            RonDeserialize(::ron::de::Error);
        }
    }
}

/// Wrapper class for things that should not be serialized.
struct Cache<T> {
    inner: T,
}

impl<T: Default> Cache<T> {
    pub fn new() -> Cache<T> { Cache { inner: Default::default() } }
}

impl<T> ::std::ops::Deref for Cache<T> {
    type Target = T;

    fn deref(&self) -> &T { &self.inner }
}

impl<T> ::std::ops::DerefMut for Cache<T> {
    fn deref_mut(&mut self) -> &mut T { &mut self.inner }
}
impl<T: Default> serde::Serialize for Cache<T> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> { ().serialize(s) }
}

impl<'a, T: Default> serde::Deserialize<'a> for Cache<T> {
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        let _: () = serde::Deserialize::deserialize(d)?;
        Ok(Cache::new())
    }
}
