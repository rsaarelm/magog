use crate::seeded_rng;
use euclid::{point2, rect, Point2D, Rect};
use num::{Float, One, Zero};
use rand::distributions::{Distribution, Standard, Uniform};
use rand::Rng;
use std::error::Error;
use std::fmt;
use std::hash::Hash;
use std::ops::{Add, AddAssign, Div, Mul, Range, RangeInclusive, Sub, SubAssign};

/// A type that represents convex bounds in space of `Self::T`.
pub trait Clamp {
    type T;

    /// Return the closest point inside the bounds to the argument.
    fn clamp(&self, point: Self::T) -> Self::T;
}

impl<T: Copy + PartialOrd> Clamp for RangeInclusive<T> {
    type T = T;

    fn clamp(&self, point: Self::T) -> Self::T {
        let (start, end) = (*self.start(), *self.end());
        if point < start {
            start
        } else if point > end {
            end
        } else {
            point
        }
    }
}

// Bit mathematically incorrect to define clamp for non-inclusive range, but we need it anyway for
// the Rect that we only have a non-inclusive version of, so gonna just go with it.
impl<T: Copy + PartialOrd> Clamp for Range<T> {
    type T = T;

    fn clamp(&self, point: Self::T) -> Self::T { (self.start..=self.end).clamp(point) }
}

// Impls of Clamp for RangeFrom and RangeToInclusive are trivial to add here if they're ever
// needed.

impl<
        T: Copy
            + Clone
            + euclid::num::Zero
            + PartialOrd
            + PartialEq
            + Add<T, Output = T>
            + Sub<T, Output = T>,
        U,
    > Clamp for Rect<T, U>
{
    type T = Point2D<T, U>;

    fn clamp(&self, point: Self::T) -> Self::T {
        point2(self.x_range().clamp(point.x), self.y_range().clamp(point.y))
    }
}

/// Deterministic noise.
///
/// Turns an input value into a noise value.
///
/// # Examples
///
/// ```
/// use rand::distributions::Uniform;
/// use calx::Noise;
///
/// let depth = Uniform::new_inclusive(-1.0, 1.0);
/// let z: f32 = depth.noise(&(12, 34));
/// assert_eq!(z, -0.6524992);
/// let z: f32 = depth.noise(&(34, 12));
/// assert_eq!(z, -0.5685262);
/// ```
pub trait Noise<T> {
    fn noise(&self, seed: &impl Hash) -> T;
}

impl<T: Distribution<U>, U> Noise<U> for T {
    fn noise(&self, seed: &impl Hash) -> U { self.sample(&mut seeded_rng(&seed)) }
}

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

impl Deciban {
    /// Build a deciban value from a probability in [0, 1).
    pub fn new(p: f32) -> Deciban {
        debug_assert!(p >= 0.0 && p < 1.0);
        Deciban(10.0 * (p / (1.0 - p)).log(10.0))
    }

    /// Convert a deciban value to the corresponding probability in [0, 1).
    pub fn to_p(self) -> f32 { 1.0 - 1.0 / (1.0 + 10.0.powf(self.0 / 10.0)) }
}

impl Distribution<Deciban> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> crate::Deciban {
        Deciban::new(rng.gen_range(0.0..1.0))
    }
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
pub fn lerp<T, U, V, W>(a: U, b: U, t: T) -> W
where
    U: Add<V, Output = W> + Sub<U, Output = V> + Copy,
    V: Mul<T, Output = V>,
{
    a + (b - a) * t
}

/// Sequence of points for linearly interpolating between them.
pub struct LerpPath<T, U> {
    points: Vec<(T, U)>,
}

impl<T, U, V, W> LerpPath<T, U>
where
    U: Add<V, Output = W> + Sub<U, Output = V> + Copy,
    V: Mul<T, Output = V>,
    T: PartialOrd + Sub<T, Output = T> + Div<T, Output = T> + Copy + Zero,
{
    pub fn new(begin: (T, U), end: (T, U)) -> LerpPath<T, U> {
        let mut result = LerpPath {
            points: vec![begin],
        };
        result.add(end);
        result
    }

    pub fn add(&mut self, point: (T, U)) {
        for i in 0..self.points.len() {
            if point.0 == self.points[i].0 {
                // Avoid zero width intervals or we'll get division by zero down the road, just
                // replace the point if the position is exactly the same.
                self.points[i] = point;
                return;
            }
            if point.0 < self.points[i].0 {
                self.points.insert(i, point);
                return;
            }
        }

        self.points.push(point);
    }

    pub fn sample(&self, t: T) -> W {
        if t < self.points[0].0 {
            return lerp(self.points[0].1, self.points[0].1, T::zero());
        }

        for i in 1..self.points.len() {
            if t < self.points[i].0 {
                let scaled = (t - self.points[i - 1].0) / (self.points[i].0 - self.points[i - 1].0);
                return lerp(self.points[i - 1].1, self.points[i].1, scaled);
            }
        }

        lerp(
            self.points[self.points.len() - 1].1,
            self.points[self.points.len() - 1].1,
            T::zero(),
        )
    }
}

/// Single-pass weighted sample for iterations.
pub trait WeightedChoice {
    type Item;

    /// Choose an item from the iteration with probability weighted by item weight.
    fn weighted_choice<R: Rng + ?Sized, F>(self, rng: &mut R, weight_fn: F) -> Option<Self::Item>
    where
        F: Fn(&Self::Item) -> f32;
}

impl<T, I: IntoIterator<Item = T> + Sized> WeightedChoice for I {
    type Item = T;

    fn weighted_choice<R: Rng + ?Sized, F>(self, rng: &mut R, weight_fn: F) -> Option<Self::Item>
    where
        F: Fn(&Self::Item) -> f32,
    {
        let dist = Uniform::new(0.0, 1.0);
        let (_, ret) = self
            .into_iter()
            .fold((0.0, None), |(weight_sum, prev_item), item| {
                let item_weight = weight_fn(&item);
                debug_assert!(item_weight >= 0.0);
                let p = item_weight / (weight_sum + item_weight);
                let next_item = if dist.sample(rng) < p {
                    Some(item)
                } else {
                    prev_item
                };
                (weight_sum + item_weight, next_item)
            });
        ret
    }
}

/// Insert a 0 bit between the low 16 bits of a number.
///
/// Useful for <https://en.wikipedia.org/wiki/Z-order_curve>
#[inline(always)]
pub fn spread_bits_by_2(mut bits: u32) -> u32 {
    // from https://fgiesen.wordpress.com/2009/12/13/decoding-morton-codes/
    bits &= /* --------------- */ 0b0000_0000_0000_0000_1111_1111_1111_1111;
    bits = (bits ^ (bits << 8)) & 0b0000_0000_1111_1111_0000_0000_1111_1111;
    bits = (bits ^ (bits << 4)) & 0b0000_1111_0000_1111_0000_1111_0000_1111;
    bits = (bits ^ (bits << 2)) & 0b0011_0011_0011_0011_0011_0011_0011_0011;
    bits = (bits ^ (bits << 1)) & 0b0101_0101_0101_0101_0101_0101_0101_0101;
    bits
}

/// Remove every odd bit and compact the even bits into the lower half of the number.
///
/// Useful for <https://en.wikipedia.org/wiki/Z-order_curve>
#[inline(always)]
pub fn compact_bits_by_2(mut bits: u32) -> u32 {
    // from https://fgiesen.wordpress.com/2009/12/13/decoding-morton-codes/
    bits &= /* --------------- */ 0b0101_0101_0101_0101_0101_0101_0101_0101;
    bits = (bits ^ (bits >> 1)) & 0b0011_0011_0011_0011_0011_0011_0011_0011;
    bits = (bits ^ (bits >> 2)) & 0b0000_1111_0000_1111_0000_1111_0000_1111;
    bits = (bits ^ (bits >> 4)) & 0b0000_0000_1111_1111_0000_0000_1111_1111;
    bits = (bits ^ (bits >> 8)) & 0b0000_0000_0000_0000_1111_1111_1111_1111;
    bits
}

/// Repeatedly run a random generator that may fail until it succeeds.
pub fn retry_gen<R: Rng + ?Sized, T, E>(
    n_tries: usize,
    rng: &mut R,
    gen: impl Fn(&mut R) -> Result<T, E>,
) -> Result<T, E> {
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
/// It will add unit distance to the right and bottom edges to make sure the resulting `Rect`
/// will contain all the points. It would beneath
///
/// # Examples
///
/// ```
/// use euclid::point2;
/// type Point2D<T> = euclid::Point2D<T, euclid::UnknownUnit>;
///
/// let points: Vec<Point2D<i32>> = vec![point2(2, 3), point2(4, 5), point2(6, 7)];
/// let rect = calx::bounding_rect(&points);
/// assert!(rect.contains(point2(2, 3)));
/// assert!(rect.contains(point2(6, 7)));
/// assert!(!rect.contains(point2(7, 7)));
/// assert!(!rect.contains(point2(2, 2)));
/// ```
pub fn bounding_rect<'a, I, T, U>(points: I) -> Rect<T, U>
where
    I: IntoIterator<Item = &'a Point2D<T, U>>,
    T: Zero + One + Add + Sub<Output = T> + Ord + Copy + 'a,
    U: 'a,
{
    let mut iter = points.into_iter();
    if let Some(first) = iter.next() {
        let (mut min_x, mut min_y) = (first.x, first.y);
        let (mut max_x, mut max_y) = (first.x, first.y);

        for p in iter {
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
        Rect::zero()
    }
}

/// Error type for when you don't care about the type and just want to use a string message.
#[derive(Debug)]
pub struct GenericError(pub String);

impl Error for GenericError {}

impl fmt::Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}

/// Construct a `GenericError`.
#[macro_export]
macro_rules! err {
    ($($arg:tt)*) => ($crate::GenericError(format!($($arg)*)));
}

/// Return a `Result<_, Box<Error>>` error with `GenericError` message.
#[macro_export]
macro_rules! die {
    ($($arg:tt)*) => (return Err(Box::new($crate::err!($($arg)*))););
}
