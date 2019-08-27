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
pub(crate) struct MessageFormatter0<'a, W> {
    world: &'a mut W,
    msg: String,
}

#[must_use]
pub(crate) struct MessageFormatter1<'a, W> {
    world: &'a mut W,
    subject: grammar::Noun,
    msg: String,
}

#[must_use]
pub(crate) struct MessageFormatter2<'a, W> {
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
        use crate::grammar::Templater;
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
        use crate::grammar::Templater;
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
        use crate::grammar::Templater;
        let event = Event::Msg(
            grammar::ObjectTemplater::new(
                grammar::SubjectTemplater::new(self.subject),
                self.object,
            )
            .format(&self.msg)
            .unwrap(),
        );
        self.world.push_event(event);
    }
}

mod animations;
pub use crate::animations::Animations;

mod biome;

mod command;
pub use crate::command::{ActionOutcome, Command};

mod components;
pub use crate::components::{Anim, AnimState, Icon};

mod effect;

mod event;
pub use crate::event::Event;

mod flags;

mod fov;

mod grammar;

mod item;
pub use crate::item::{ItemType, Slot};

mod location;
pub use crate::location::{Location, Portal};

mod location_set;

mod mapsave;
pub use crate::mapsave::MapSave;

mod map;

mod mutate;

mod query;
pub use crate::query::Query;

mod sector;
pub use crate::sector::{Sector, SECTOR_HEIGHT, SECTOR_WIDTH};

mod spatial;
mod spec;
mod stats;

mod terraform;
pub use crate::terraform::{Terraform, TerrainQuery};

pub mod terrain;
pub use crate::terrain::Terrain;

mod vaults;

mod volume;

mod world;
pub use crate::world::{Ecs, World};

mod worldgen;

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
    (weapon_power as f32 * calx::clamp(0.0, MAX_DAMAGE_MULTIPLIER, (roll - 2.0) * 0.05)) as i32
}

/// Standard deciban roll, clamp into [-20, 20].
pub fn roll(rng: &mut impl rand::Rng) -> f32 { calx::clamp(-20.0, 20.0, rng.gen::<Deciban>().0) }
