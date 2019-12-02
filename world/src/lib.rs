use calx::{Clamp, Deciban};

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
// XXX: W should be some trait that just provides the "push message" method
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

impl<'a> MessageFormatter0<'a, World> {
    pub fn new(
        world: &'a mut World,
        msg: String,
    ) -> MessageFormatter0<'a, World> {
        MessageFormatter0 { world, msg }
    }

    pub fn subject(self, e: calx_ecs::Entity) -> MessageFormatter1<'a, World> {
        let subject = self.world.noun(e);
        MessageFormatter1 {
            world: self.world,
            subject,
            msg: self.msg,
        }
    }

    pub fn send(self) {
        use crate::grammar::Templater;
        let event =
            Event::Msg(grammar::EmptyTemplater.format(&self.msg).unwrap());
        self.world.push_event(event);
    }
}

impl<'a> MessageFormatter1<'a, World> {
    pub fn object(self, e: calx_ecs::Entity) -> MessageFormatter2<'a, World> {
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

impl<'a> MessageFormatter2<'a, World> {
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

mod ai;

mod animations;
pub use animations::{Anim, AnimState, LerpLocation};

mod command;
pub use command::{ActionOutcome, Command};

mod components;
pub use components::Icon;

mod effect;
pub use effect::Ability;

mod event;
pub use event::Event;

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

mod mutate;

mod query;

mod sector;
pub use sector::{Sector, WorldSkeleton, SECTOR_HEIGHT, SECTOR_WIDTH};

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
    (weapon_power as f32
        * (0.0..=MAX_DAMAGE_MULTIPLIER).clamp((roll - 2.0) * 0.05)) as i32
}

/// Standard deciban roll, clamp into [-20, 20].
pub fn roll(rng: &mut impl rand::Rng) -> f32 {
    (-20.0..=20.0).clamp(rng.gen::<Deciban>().0)
}
