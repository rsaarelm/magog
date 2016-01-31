#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]
extern crate serde;
extern crate num;
extern crate rand;

use std::ops::{Add, Sub, Mul};
use num::Float;

pub use rng::{EncodeRng, RngExt};
pub use text::{split_line, wrap_lines, Map2DIterator, Map2DUtil};

mod rng;
mod text;

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
    let m = (n * (n * n * Wrapping(15731) + Wrapping(789221)) +
             Wrapping(1376312589)) & Wrapping(0x7fffffff);
    let Wrapping(m) = m;
    1.0 - m as f32 / 1073741824.0
}

/// Convert probability to a log odds deciban value.
///
/// Log odds correspond to the Bayesian probability for a hypothesis that
/// has decibans * 1/10 log_2(10) bits of evidence in favor of it. They're
/// a bit like rolling a d20 but better.
pub fn to_log_odds(p: f32) -> f32 {
    10.0 * (p / (1.0 - p)).log(10.0)
}

/// Convert a log odds deciban value to the corresponding probability.
pub fn from_log_odds(db: f32) -> f32 {
    (1.0 - 1.0 / (1.0 + 10.0.powf(db / 10.0)))
}

/// Interpolate linearly between two values.
pub fn lerp<T, U>(a: U, b: U, t: T) -> U
    where U: Add<U, Output = U> + Sub<U, Output = U> + Mul<T, Output = U> + Copy
{
    a + (b - a) * t
}

/// Return the two arguments sorted to order.
pub fn sorted_pair<T: PartialOrd>(a: T, b: T) -> (T, T) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}
