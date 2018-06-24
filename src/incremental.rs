use serde::de::Error;
use serde::{self, Deserialize, Serialize};
use std::fmt::Debug;
use std::mem;
use std::ops::Deref;

/// An object that can be incrementally updated with events.
pub trait Incremental: Sized {
    /// Incoming event type.
    type Event;
    /// Error in case an event couldn't be handled.
    type Error: Debug;

    /// Try to update the object with an event.
    fn update(self, e: &Self::Event) -> Result<Self, Self::Error>;
}

/// Handle that stores an `Incremental` object and the event sequence used to build it.
///
/// Serializes itself by only storing the log.
pub struct IncrementalState<T: Incremental + Default> {
    log: Vec<T::Event>,
    state: T,
}

impl<T: Incremental + Default> Default for IncrementalState<T> {
    fn default() -> Self {
        IncrementalState {
            log: Vec::new(),
            state: Default::default(),
        }
    }
}

impl<T: Incremental + Default> IncrementalState<T> {
    pub fn push(&mut self, e: T::Event) -> Result<(), T::Error> {
        match self.update(&e) {
            Ok(()) => {
                self.log.push(e);
                Ok(())
            }
            Err(e) => {
                // Uh-oh. Update crapped out and state just got eaten. Have to replay to get back
                // to where we were. XXX: This can be very expensive.
                //
                // (The state doesn't have a &mut self update method because you can't really
                // enforce the implication that an update that results an error shouldn't change
                // the object in any way. Maybe instead of automatically running the replay a
                // failed IncrementalState::push should return a core dump object that
                // has the log up to the point of failure, and the failing event. A new state could
                // then be constructed from the dump.)
                self.replay();
                Err(e)
            }
        }
    }

    pub fn pop(&mut self) -> Option<T::Event> {
        let ret = self.log.pop();
        self.replay();
        ret
    }

    fn update(&mut self, e: &T::Event) -> Result<(), T::Error> {
        let mut current_state = Default::default();
        mem::swap(&mut current_state, &mut self.state);

        match current_state.update(&e) {
            Ok(s) => {
                self.state = s;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn replay(&mut self) {
        let mut state: T = Default::default();

        for e in &self.log {
            // We already verified the update once in `push`, so if it fails now that's a bug
            // in the state type and causes a panic here.
            state = state.update(e).unwrap();
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
        let mut ret: IncrementalState<T> = Default::default();
        for e in Vec::deserialize(d)?.into_iter() {
            match ret.push(e) {
                Ok(()) => {}
                Err(e) => return Err(D::Error::custom(format!("{:?}", e))),
            }
        }
        Ok(ret)
    }
}
