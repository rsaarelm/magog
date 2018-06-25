use serde::de::Error;
use serde::{self, Deserialize, Serialize};
use std::fmt::Debug;
use std::mem;
use std::ops::Deref;

/// An object that can be incrementally updated with events.
pub trait Incremental: Sized {
    /// Incoming event type.
    type Event;

    /// Try to update the object with an event.
    fn update(self, e: &Self::Event) -> Self;
}

/// An incremental object that can fail when updated.
///
/// Use the `Incremental` implementation on a `Result` type with `FailingIncremental` value to use
/// this.
pub trait FailingIncremental: Sized {
    /// Incoming event type.
    type Event;
    /// Error in case an event couldn't be handled.
    type Error: Debug;

    /// Try to update the object with an event.
    fn update(self, e: &Self::Event) -> Result<Self, Self::Error>;
}

impl<T: FailingIncremental> Incremental for Result<T, T::Error> {
    type Event = T::Event;

    fn update(self, e: &Self::Event) -> Self {
        match self {
            Ok(state) => state.update(e),
            err => err,
        }
    }
}

/// Handle that stores an `Incremental` object and the event sequence used to build it.
///
/// Serializes itself by only storing the log.
pub struct IncrementalState<T: Incremental + Default> {
    log: Vec<T::Event>,
    state: T,
}

// Implemented manually instead of derived so that T::Event doesn't need to implement Default.
impl<T: Incremental + Default> Default for IncrementalState<T> {
    fn default() -> Self {
        IncrementalState {
            log: Vec::new(),
            state: Default::default(),
        }
    }
}

impl<T: Incremental + Default> IncrementalState<T> {
    pub fn push(&mut self, e: T::Event) {
        // XXX: Need to get state away from borrow checker for updating, is there a better way than
        // ugly unsafe tricks?
        let state = mem::replace(&mut self.state, unsafe { mem::uninitialized() }).update(&e);
        self.state = state;
        self.log.push(e);
    }

    pub fn pop(&mut self) -> Option<T::Event> {
        // TODO: Way to pop several items without O(n) delay for each.
        let ret = self.log.pop();
        self.replay();
        ret
    }

    fn replay(&mut self) {
        let mut state: T = Default::default();

        for e in &self.log {
            state = state.update(e);
        }

        self.state = state;
    }
}

impl<T: Incremental + Default> Deref for IncrementalState<T> {
    type Target = T;

    fn deref(&self) -> &T { &self.state }
}

impl<T: Incremental<Event = impl Serialize> + Default> Serialize for IncrementalState<T> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.log.serialize(s)
    }
}

impl<'a, T: Incremental<Event = impl Deserialize<'a>> + Default> Deserialize<'a>
    for IncrementalState<T>
{
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        let mut ret: Self = IncrementalState {
            log: Vec::deserialize(d)?,
            state: Default::default(),
        };
        ret.replay();
        Ok(ret)
    }
}
