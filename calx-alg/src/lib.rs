extern crate num;
extern crate rand;
extern crate time;
extern crate rustc_serialize;

use rand::Rng;
use std::ops::{Add, Mul, Sub};
use num::Float;

pub use rng::{EncodeRng, RngExt};
pub use text::{Map2DIterator, Map2DUtil, split_line, wrap_lines};

mod rng;
mod text;
pub mod timing;

pub mod ease;

/// Clamp a value to range.
pub fn clamp<C: PartialOrd + Copy>(mn: C, mx: C, x: C) -> C {
    if x < mn {
        mn
    } else if x > mx {
        mx
    } else {
        x
    }
}

/// Deterministic noise.
pub fn noise(n: i32) -> f32 {
    use std::num::Wrapping;

    let n = Wrapping(n);
    let n = (n << 13) ^ n;
    let m = (n * (n * n * Wrapping(15731) + Wrapping(789221)) + Wrapping(1376312589)) &
            Wrapping(0x7fffffff);
    let Wrapping(m) = m;
    1.0 - m as f32 / 1073741824.0
}

/// Convert probability to a log odds deciban value.
///
/// Log odds correspond to the Bayesian probability for a hypothesis that
/// has decibans * 1/10 log_2(10) bits of evidence in favor of it. They're
/// a bit like rolling a d20 but better.
///
/// # Examples
///
/// ```
/// use calx_alg::to_log_odds;
/// assert_eq!(0.0, to_log_odds(0.5));
/// assert_eq!(10, to_log_odds(0.909091) as i32);
/// ```
pub fn to_log_odds(p: f32) -> f32 { 10.0 * (p / (1.0 - p)).log(10.0) }

/// Convert a log odds deciban value to the corresponding probability.
///
/// # Examples
///
/// ```
/// use calx_alg::from_log_odds;
/// assert_eq!(0.5, from_log_odds(0.0));
/// assert_eq!(24, (from_log_odds(-5.0) * 100.0) as i32);
/// ```
pub fn from_log_odds(db: f32) -> f32 { (1.0 - 1.0 / (1.0 + 10.0.powf(db / 10.0))) }

/// Interpolate linearly between two values.
pub fn lerp<T, U>(a: U, b: U, t: T) -> U
    where U: Add<U, Output = U> + Sub<U, Output = U> + Mul<T, Output = U> + Copy
{
    a + (b - a) * t
}

/// Return the two arguments sorted to order.
pub fn sorted_pair<T: PartialOrd>(a: T, b: T) -> (T, T) { if a < b { (a, b) } else { (b, a) } }

// TODO: Remove this thing once Rust has a proper way of counting macro
// arguments.

/// Macro hack for counting the number of arguments.
///
/// Use for determining the size of a fixed array type created by a macro that
/// takes variadic arguments. Due to its hackiness, will fail around 100
/// elements.
#[macro_export]
macro_rules! count_exprs {
    () => { 0 };
    ($e:expr) => { 1 };
    ($e:expr, $($es:expr),+) => { 1 + count_exprs!($($es),*) };
}

/// Single-pass weighted sample for iterations.
pub trait WeightedChoice {
    type Item;

    /// Choose an item from the iteration with probability weighted by item weight.
    fn weighted_choice<R: Rng, F>(self, rng: &mut R, weight_fn: F) -> Option<Self::Item>
        where F: Fn(&Self::Item) -> f32;
}

impl<T, I: Iterator<Item = T> + Sized> WeightedChoice for I {
    type Item = T;

    fn weighted_choice<R: Rng, F>(self, rng: &mut R, weight_fn: F) -> Option<Self::Item>
        where F: Fn(&Self::Item) -> f32
    {
        self.fold((0.0, None), |(weight_sum, prev_item), item| {
                let item_weight = weight_fn(&item);
                assert!(item_weight >= 0.0);
                let p = item_weight / (weight_sum + item_weight);
                let next_item = if rng.next_f32() < p { Some(item) } else { prev_item };
                (weight_sum + item_weight, next_item)
            })
            .1
    }
}
