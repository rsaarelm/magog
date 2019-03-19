use serde;
use serde_derive::{Deserialize, Serialize};
use std::mem;
use std::ops::Deref;

/// An object that can be incrementally updated with events.
pub trait Incremental: Sized {
    type Seed;
    type Event;

    /// Initialize the starting state from a seed value.
    fn from_seed(s: &Self::Seed) -> Self;

    /// Try to update the object with an event.
    fn update(self, e: &Self::Event) -> Self;
}

/// Handle that stores an `Incremental` object and the event sequence used to build it.
///
/// The full `IncrementalState` serializes itself normally with a full snapshot of the state, but
/// you can make a possibly much smaller serialization by just saving the History and then creating
/// a new state with `IncrmentalState::from(history)`. This will involve replaying the entire
/// history so it might be much slower than just deserializing the snapshot.
///
/// # Examples
///
/// ```
/// use calx::{Incremental, IncrementalState};
///
/// #[derive(Default)]
/// struct State(u32);
///
/// impl Incremental for State {
///     type Seed = u32;
///     type Event = u32;
///
///     fn update(self, e: &Self::Event) -> State { State(self.0 + *e) }
///     fn from_seed(s: &Self::Seed) -> Self { State(*s) }
/// }
///
/// let mut handle: IncrementalState<State> = Default::default();
/// assert_eq!(handle.0, 0);
///
/// for x in &[1, 2, 3, 4] { handle.update(*x); }
/// assert_eq!(handle.0, 10);
///
/// // Save the history.
/// let history_copy = handle.history().clone();
///
/// handle.undo();               // Magical time travel!
/// assert_eq!(handle.0, 6);
///
/// // Create a new state just from the saved history.
/// let handle2: IncrementalState<State> = history_copy.into();
/// assert_eq!(handle2.0, 10);
/// ```
#[derive(Serialize, Deserialize)]
// Need to add extra boilerplate or serde will get confused by the dependenent types.
#[serde(bound(
    serialize = "T: serde::Serialize, T::Seed: serde::Serialize, T::Event: serde::Serialize",
    deserialize = "T: serde::Deserialize<'de>, T::Seed: serde::Deserialize<'de>, T::Event: serde::Deserialize<'de>",
))]
pub struct IncrementalState<T: Incremental> {
    state: T,
    history: History<T::Seed, T::Event>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct History<U, E> {
    pub seed: U,
    pub events: Vec<E>,
}

// Implemented manually instead of derived so that T::Event doesn't need to implement Default.
impl<T: Incremental<Seed = U>, U: Default> Default for IncrementalState<T> {
    fn default() -> Self { IncrementalState::new(T::Seed::default()) }
}

impl<T: Incremental> IncrementalState<T> {
    pub fn new(seed: T::Seed) -> Self {
        let state = T::from_seed(&seed);
        IncrementalState {
            state,
            history: History {
                seed,
                events: Vec::new(),
            },
        }
    }

    pub fn history(&self) -> &History<T::Seed, T::Event> { &self.history }

    pub fn update(&mut self, e: T::Event) {
        // XXX: Need to get state away from borrow checker for updating, is there a better way than
        // ugly unsafe tricks?
        let state = mem::replace(&mut self.state, unsafe { mem::uninitialized() }).update(&e);
        self.state = state;
        self.history.events.push(e);
    }

    /// Do arbitrary modification on event history, then replay state to match new history.
    ///
    /// Use this if you want to undo multiple steps, replaying can be very expensive.
    pub fn edit_history<F: FnOnce(&mut History<T::Seed, T::Event>) -> U, U>(&mut self, f: F) -> U {
        let ret = f(&mut self.history);
        self.replay();
        ret
    }

    /// Cancel the last event and rewind state to the position before it.
    pub fn undo(&mut self) -> Option<T::Event> { self.edit_history(|h| h.events.pop()) }

    fn replay(&mut self) {
        let mut state = T::from_seed(&self.history.seed);

        for e in &self.history.events {
            state = state.update(e);
        }

        self.state = state;
    }
}

impl<T: Incremental> From<History<T::Seed, T::Event>> for IncrementalState<T> {
    fn from(history: History<T::Seed, T::Event>) -> Self {
        let state = T::from_seed(&history.seed);
        let mut ret = IncrementalState { state, history };
        ret.replay();
        ret
    }
}

impl<T: Incremental> Deref for IncrementalState<T> {
    type Target = T;

    fn deref(&self) -> &T { &self.state }
}
