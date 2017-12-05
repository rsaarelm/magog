extern crate calx;
#[macro_use]
extern crate calx_ecs;
#[macro_use]
extern crate error_chain;
extern crate euclid;
extern crate image;
#[macro_use]
extern crate lazy_static;
extern crate rand;
extern crate ron;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use calx::Deciban;

/// Helper macro for formatting textual event messages.
macro_rules! msg {
    ($ctx: expr, $fmt:expr) => {
        $crate::MessageFormatter0::new($ctx, $fmt.to_string())
    };

    ($ctx: expr, $fmt:expr, $($arg:expr),*) => {
        {
            let __txt = format!($fmt, $($arg),*);
            $crate::MessageFormatter0::new($ctx, __txt)
        }
    };
}

// XXX: Lots of code for pretty simple end result on the MessageFormatter...
#[must_use]
pub struct MessageFormatter0<'a, W: 'a> {
    world: &'a mut W,
    msg: String,
}

#[must_use]
pub struct MessageFormatter1<'a, W: 'a> {
    world: &'a mut W,
    subject: grammar::Noun,
    msg: String,
}

#[must_use]
pub struct MessageFormatter2<'a, W: 'a> {
    world: &'a mut W,
    subject: grammar::Noun,
    object: grammar::Noun,
    msg: String,
}

impl<'a, W: mutate::Mutate> MessageFormatter0<'a, W> {
    pub fn new(world: &'a mut W, msg: String) -> MessageFormatter0<'a, W> {
        MessageFormatter0 { world, msg }
    }

    pub fn subject(self, e: calx_ecs::Entity) -> MessageFormatter1<'a, W> {
        let subject = self.world.noun(e);
        MessageFormatter1 {
            world: self.world,
            subject,
            msg: self.msg,
        }
    }

    pub fn send(self) {
        use grammar::Templater;
        let event = Event::Msg(grammar::EmptyTemplater.format(&self.msg).unwrap());
        self.world.push_event(event);
    }
}

impl<'a, W: mutate::Mutate> MessageFormatter1<'a, W> {
    pub fn object(self, e: calx_ecs::Entity) -> MessageFormatter2<'a, W> {
        let object = self.world.noun(e);
        MessageFormatter2 {
            world: self.world,
            subject: self.subject,
            object,
            msg: self.msg,
        }
    }

    pub fn send(self) {
        use grammar::Templater;
        let event = Event::Msg(
            grammar::SubjectTemplater::new(self.subject)
                .format(&self.msg)
                .unwrap(),
        );
        self.world.push_event(event);
    }
}

impl<'a, W: mutate::Mutate> MessageFormatter2<'a, W> {
    pub fn send(self) {
        use grammar::Templater;
        let event = Event::Msg(
            grammar::ObjectTemplater::new(
                grammar::SubjectTemplater::new(self.subject),
                self.object,
            ).format(&self.msg)
                .unwrap(),
        );
        self.world.push_event(event);
    }
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
pub use item::{ItemType, Slot};

mod location;
pub use location::{Location, Portal, Sector, SECTOR_HEIGHT, SECTOR_WIDTH};

mod location_set;

mod mapfile;
pub use mapfile::{load_prefab, save_prefab};

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
pub type Prefab = calx::Prefab<(Terrain, Vec<String>)>;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FovStatus {
    Seen,
    Remembered,
}

pub type Rng = calx::EncodeRng<rand::XorShiftRng>;

/// The combat formula.
///
/// Given a deciban roll and the relevant stats, determine amount of damage dealt.
/// Advantage is attacker skill - target defense.
pub fn attack_damage(roll: f32, advantage: i32, weapon_power: i32) -> i32 {
    const MAX_DAMAGE_MULTIPLIER: f32 = 4.0;

    let roll = roll + advantage as f32;
    (weapon_power as f32 * calx::clamp(0.0, MAX_DAMAGE_MULTIPLIER, (roll - 2.0) * 0.05)) as i32
}

/// Standard deciban roll, clamp into [-20, 20].
pub fn roll<R: rand::Rng>(rng: &mut R) -> f32 { calx::clamp(-20.0, 20.0, rng.gen::<Deciban>().0) }

pub mod errors {
    error_chain! {
        foreign_links {
            Io(::std::io::Error);
            RonSerialize(::ron::ser::Error);
            RonDeserialize(::ron::de::Error);
            PrefabError(::calx::PrefabError);
        }
    }
}

/// Wrapper class for things that should not be serialized.
struct Cache<T> {
    inner: T,
}

impl<T: Default> Cache<T> {
    pub fn new() -> Cache<T> {
        Cache {
            inner: Default::default(),
        }
    }
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
