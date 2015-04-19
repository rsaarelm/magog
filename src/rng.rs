use rand::{Rng};
use ::{to_log_odds};

/// Additional methods for random number generators.
pub trait RngExt {
    /// Return true with 50 % probability.
    fn coinflip(&mut self) -> bool;

    /// Return true with probability 1 / n.
    fn one_chance_in(&mut self, n: u32) -> bool;

    /// Return true with p probability.
    fn with_chance(&mut self, p: f32) -> bool;

    /// Return a log odds deciban score that corresponds to a random
    /// probability from [0, 1].
    ///
    fn log_odds(&mut self) -> f32;

    /// Return true with the probability corresponding to the log odds with
    /// the given deciban value.
    fn with_log_odds(&mut self, db: f32) -> bool;
}

impl<T: Rng> RngExt for T {
    fn coinflip(&mut self) -> bool { self.gen_weighted_bool(2) }

    fn one_chance_in(&mut self, n: u32) -> bool { self.gen_weighted_bool(n) }

    fn with_chance(&mut self, p: f32) -> bool {
        self.gen_range(0.0, 1.0) < p
    }

    fn log_odds(&mut self) -> f32 { to_log_odds(self.gen_range(0.0, 1.0)) }

    fn with_log_odds(&mut self, db: f32) -> bool {
        db > self.log_odds()
    }
}
