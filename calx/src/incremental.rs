use serde::{self, Deserialize, Serialize};
use std::fmt::Debug;
use std::mem;
use std::ops::Deref;

/// An object that can be incrementally updated with events.
pub trait Incremental<E>: Sized {
    /// Try to update the object with an event.
    fn update(self, e: &E) -> Self;
}

/// An incremental object that can fail when updated.
///
/// Use the `Incremental` implementation on a `Result` type with `FailingIncremental` value to use
/// this.
pub trait FailingIncremental<E>: Sized {
    /// Error in case an event couldn't be handled.
    type Error: Debug;

    /// Try to update the object with an event.
    fn update(self, e: &E) -> Result<Self, Self::Error>;
}

impl<T: FailingIncremental<E>, E> Incremental<E> for Result<T, T::Error> {
    fn update(self, e: &E) -> Self {
        match self {
            Ok(state) => state.update(e),
            err => err,
        }
    }
}

/// Handle that stores an `Incremental` object and the event sequence used to build it.
///
/// Serializes itself by only storing the log.
///
/// # Examples
///
/// ```
/// use calx::{Incremental, IncrementalState};
///
/// #[derive(Default)]
/// struct State(u32);
///
/// impl Incremental<u32> for State {
///     fn update(self, e: &u32) -> State { State(self.0 + *e) }
/// }
///
/// let mut handle: IncrementalState<State, u32> = Default::default();
/// assert_eq!(handle.0, 0);
///
/// for x in &[1, 2, 3, 4] { handle.push(*x); }
/// assert_eq!(handle.0, 10);
/// handle.pop();
/// assert_eq!(handle.0, 6);
/// ```
pub struct IncrementalState<T: Incremental<E> + Default, E> {
    log: Vec<E>,
    state: T,
}

// Implemented manually instead of derived so that T::Event doesn't need to implement Default.
impl<T: Incremental<E> + Default, E> Default for IncrementalState<T, E> {
    fn default() -> Self {
        IncrementalState {
            log: Vec::new(),
            state: Default::default(),
        }
    }
}

impl<T: Incremental<E> + Default, E> IncrementalState<T, E> {
    pub fn push(&mut self, e: E) {
        // XXX: Need to get state away from borrow checker for updating, is there a better way than
        // ugly unsafe tricks?
        let state = mem::replace(&mut self.state, unsafe { mem::uninitialized() }).update(&e);
        self.state = state;
        self.log.push(e);
    }

    pub fn pop(&mut self) -> Option<E> {
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

impl<T: Incremental<E> + Default, E> Deref for IncrementalState<T, E> {
    type Target = T;

    fn deref(&self) -> &T { &self.state }
}

impl<T: Incremental<E> + Default, E: Serialize> Serialize for IncrementalState<T, E> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.log.serialize(s)
    }
}

impl<'a, T: Incremental<E> + Default, E: Deserialize<'a>> Deserialize<'a>
    for IncrementalState<T, E>
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
