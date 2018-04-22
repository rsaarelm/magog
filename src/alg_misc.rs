
use euclid::{rect, TypedPoint2D, TypedRect};
use num::{Float, One, Zero};
use rand::{Rand, Rng};
use seeded_rng;
use std::hash::Hash;
use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

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
///
/// Turns an input value into a noise value.
///
/// # Examples
///
///     let z: u32 = calx::noise(&(12, 34));
///     assert_eq!(z, 746252712);
///     let z: u32 = calx::noise(&(34, 12));
///     assert_eq!(z, 926582979);
pub fn noise<I: Hash, O: Rand>(seed: &I) -> O { seeded_rng(&seed).gen() }

/// A deciban unit log odds value.
///
/// Log odds correspond to the Bayesian probability for a hypothesis that
/// has decibans * 1/10 log_2(10) bits of evidence in favor of it. They're
/// a bit like rolling a d20 but better.
///
/// # Examples
///
/// ```
/// use calx::Deciban;
/// assert_eq!(0.0, Deciban::new(0.5).0);
/// assert_eq!(10, Deciban::new(0.909091).0 as i32);
///
/// assert_eq!(0.5, Deciban(0.0).to_p());
/// assert_eq!(24, (Deciban(-5.0).to_p() * 100.0) as i32);
/// ```
#[derive(Copy, Clone, PartialEq, PartialOrd, Default, Debug)]
pub struct Deciban(pub f32);

impl Rand for Deciban {
    fn rand<R: Rng>(rng: &mut R) -> Self { Deciban::new(rng.next_f32()) }
}

impl Deciban {
    /// Build a deciban value from a probability in [0, 1).
    pub fn new(p: f32) -> Deciban {
        debug_assert!(p >= 0.0 && p < 1.0);
        Deciban(10.0 * (p / (1.0 - p)).log(10.0))
    }

    /// Convert a deciban value to the corresponding probability in [0, 1).
    pub fn to_p(self) -> f32 { 1.0 - 1.0 / (1.0 + 10.0.powf(self.0 / 10.0)) }
}

impl Add for Deciban {
    type Output = Deciban;
    fn add(self, rhs: Deciban) -> Deciban { Deciban(self.0 + rhs.0) }
}

impl AddAssign for Deciban {
    fn add_assign(&mut self, rhs: Deciban) { self.0 += rhs.0; }
}

impl Sub for Deciban {
    type Output = Deciban;
    fn sub(self, rhs: Deciban) -> Deciban { Deciban(self.0 - rhs.0) }
}

impl SubAssign for Deciban {
    fn sub_assign(&mut self, rhs: Deciban) { self.0 -= rhs.0; }
}

/// Interpolate linearly between two values.
pub fn lerp<T, U>(a: U, b: U, t: T) -> U
where
    U: Add<U, Output = U> + Sub<U, Output = U> + Mul<T, Output = U> + Copy,
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
    where
        F: Fn(&Self::Item) -> f32;
}

impl<T, I: IntoIterator<Item = T> + Sized> WeightedChoice for I {
    type Item = T;

    fn weighted_choice<R: Rng, F>(self, rng: &mut R, weight_fn: F) -> Option<Self::Item>
    where
        F: Fn(&Self::Item) -> f32,
    {
        let (_, ret) = self.into_iter().fold(
            (0.0, None),
            |(weight_sum, prev_item), item| {
                let item_weight = weight_fn(&item);
                debug_assert!(item_weight >= 0.0);
                let p = item_weight / (weight_sum + item_weight);
                let next_item = if rng.next_f32() < p {
                    Some(item)
                } else {
                    prev_item
                };
                (weight_sum + item_weight, next_item)
            },
        );
        ret
    }
}

/// Insert a 0 bit between the low 16 bits of a number.
///
/// Useful for <https://en.wikipedia.org/wiki/Z-order_curve>
#[inline(always)]
pub fn spread_bits_by_2(mut bits: u32) -> u32 {
    // from https://fgiesen.wordpress.com/2009/12/13/decoding-morton-codes/
    bits &= /* --------------- */ 0b00000000_00000000_11111111_11111111;
    bits = (bits ^ (bits << 8)) & 0b00000000_11111111_00000000_11111111;
    bits = (bits ^ (bits << 4)) & 0b00001111_00001111_00001111_00001111;
    bits = (bits ^ (bits << 2)) & 0b00110011_00110011_00110011_00110011;
    bits = (bits ^ (bits << 1)) & 0b01010101_01010101_01010101_01010101;
    bits
}

/// Remove every odd bit and compact the even bits into the lower half of the number.
///
/// Useful for <https://en.wikipedia.org/wiki/Z-order_curve>
#[inline(always)]
pub fn compact_bits_by_2(mut bits: u32) -> u32 {
    // from https://fgiesen.wordpress.com/2009/12/13/decoding-morton-codes/
    bits &= /* --------------- */ 0b01010101_01010101_01010101_01010101;
    bits = (bits ^ (bits >> 1)) & 0b00110011_00110011_00110011_00110011;
    bits = (bits ^ (bits >> 2)) & 0b00001111_00001111_00001111_00001111;
    bits = (bits ^ (bits >> 4)) & 0b00000000_11111111_00000000_11111111;
    bits = (bits ^ (bits >> 8)) & 0b00000000_00000000_11111111_11111111;
    bits
}

/// Repeatedly run a random generator that may fail until it succeeds.
pub fn retry_gen<R: Rng, F, T, E>(n_tries: usize, rng: &mut R, gen: F) -> Result<T, E>
where
    F: Fn(&mut R) -> Result<T, E>,
{
    let mut ret = gen(rng);
    for _ in 0..(n_tries - 1) {
        if ret.is_ok() {
            return ret;
        }
        ret = gen(rng);
    }

    ret
}

/// Build an axis-aligned rectangle that contains the given points.
///
/// It will add unit distance to the right and bottom edges to make sure the resulting `TypedRect`
/// will contain all the points. It would beneath
///
/// # Examples
///
/// ```
/// # extern crate euclid;
/// # extern crate calx;
/// # fn main() {
/// use euclid::point2;
///
/// let points: Vec<euclid::Point2D<i32>> = vec![point2(2, 3), point2(4, 5), point2(6, 7)];
/// let rect = calx::bounding_rect(&points);
/// assert!(rect.contains(&point2(2, 3)));
/// assert!(rect.contains(&point2(6, 7)));
/// assert!(!rect.contains(&point2(7, 7)));
/// assert!(!rect.contains(&point2(2, 2)));
/// # }
/// ```
pub fn bounding_rect<'a, I, T, U>(points: I) -> TypedRect<T, U>
where
    I: IntoIterator<Item = &'a TypedPoint2D<T, U>>,
    T: Zero + One + Add + Sub<Output = T> + Ord + Copy + 'a,
    U: 'a,
{
    let mut iter = points.into_iter();
    if let Some(first) = iter.next() {
        let (mut min_x, mut min_y) = (first.x, first.y);
        let (mut max_x, mut max_y) = (first.x, first.y);

        while let Some(p) = iter.next() {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }

        // Make the rect contain the edge points.
        max_x = max_x + T::one();
        max_y = max_y + T::one();

        rect(min_x, min_y, max_x - min_x, max_y - min_y)
    } else {
        TypedRect::zero()
    }
}
