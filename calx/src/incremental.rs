use serde;
use serde_derive::{Deserialize, Serialize};
use std::mem;
use std::ops::Deref;

/// An object that can be incrementally updated with events.
pub trait Incremental<E>: Sized {
    /// Try to update the object with an event.
    fn update(self, e: &E) -> Self;
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
/// impl From<u32> for State {
///     fn from(x: u32) -> Self { State(x) }
/// }
///
/// let mut handle: IncrementalState<State, u32, u32> = Default::default();
/// assert_eq!(handle.0, 0);
///
/// for x in &[1, 2, 3, 4] { handle.push(*x); }
/// assert_eq!(handle.0, 10);
/// handle.pop();
/// assert_eq!(handle.0, 6);
/// ```
pub struct IncrementalState<T: Incremental<E>, U: Into<T> + Clone, E> {
    state: T,
    history: History<U, E>,
}

#[derive(Serialize, Deserialize)]
struct History<U, E> {
    seed: U,
    log: Vec<E>,
}

// Implemented manually instead of derived so that T::Event doesn't need to implement Default.
impl<T: Incremental<E>, U: Into<T> + Clone + Default, E> Default for IncrementalState<T, U, E> {
    fn default() -> Self { IncrementalState::new(U::default()) }
}

impl<T: Incremental<E>, U: Into<T> + Clone, E> IncrementalState<T, U, E> {
    pub fn new(seed: U) -> Self {
        IncrementalState {
            state: seed.clone().into(),
            history: History {
                seed,
                log: Vec::new(),
            },
        }
    }
}

impl<T: Incremental<E>, U: Into<T> + Clone, E> IncrementalState<T, U, E> {
    pub fn push(&mut self, e: E) {
        // XXX: Need to get state away from borrow checker for updating, is there a better way than
        // ugly unsafe tricks?
        let state = mem::replace(&mut self.state, unsafe { mem::uninitialized() }).update(&e);
        self.state = state;
        self.history.log.push(e);
    }
}

impl<T: Incremental<E>, U: Into<T> + Clone, E> IncrementalState<T, U, E> {
    pub fn pop(&mut self) -> Option<E> {
        // TODO: Way to pop several items without O(n) delay for each.
        let ret = self.history.log.pop();
        self.replay();
        ret
    }

    fn replay(&mut self) {
        let mut state: T = self.history.seed.clone().into();

        for e in &self.history.log {
            state = state.update(e);
        }

        self.state = state;
    }
}

impl<T: Incremental<E>, U: Into<T> + Clone, E> Deref for IncrementalState<T, U, E> {
    type Target = T;

    fn deref(&self) -> &T { &self.state }
}

impl<T, U, E> serde::Serialize for IncrementalState<T, U, E>
where
    T: Incremental<E>,
    U: Into<T> + Clone + serde::Serialize,
    E: serde::Serialize,
{
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.history.serialize(s)
    }
}

impl<'a, T, U, E> serde::Deserialize<'a> for IncrementalState<T, U, E>
where
    T: Incremental<E>,
    U: Into<T> + Clone + serde::Deserialize<'a>,
    E: serde::Deserialize<'a>,
{
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        let history: History<U, E> = History::deserialize(d)?;
        let state = history.seed.clone().into();
        let mut ret: Self = IncrementalState { state, history };
        ret.replay();
        Ok(ret)
    }
}
