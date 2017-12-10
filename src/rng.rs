use Deciban;
use rand::{Rng, SeedableRng};
use serde;
use std::mem;
use vec_map::VecMap;

/// Additional methods for random number generators.
pub trait RngExt {
    /// Return true with 50 % probability.
    fn coinflip(&mut self) -> bool;

    /// Return true with probability 1 / n.
    fn one_chance_in(&mut self, n: u32) -> bool;

    /// Return true with p probability.
    fn with_chance(&mut self, p: f32) -> bool;

    /// Return true with the probability corresponding to the log odds with
    /// the given deciban value.
    fn with_log_odds(&mut self, db: Deciban) -> bool;
}

impl<T: Rng> RngExt for T {
    fn coinflip(&mut self) -> bool { self.gen_weighted_bool(2) }

    fn one_chance_in(&mut self, n: u32) -> bool { self.gen_weighted_bool(n) }

    fn with_chance(&mut self, p: f32) -> bool { self.gen_range(0.0, 1.0) < p }

    fn with_log_odds(&mut self, db: Deciban) -> bool { db > self.gen::<Deciban>() }
}

/// A wrapper that makes a Rng implementation encodable.
///
/// For games that want to store the current Rng state as a part of the save
/// game. Works by casting the Rng representation into a binary blob, will
/// crash and burn if the Rng struct is not plain-old-data.
#[derive(Clone, Debug)]
pub struct EncodeRng<T> {
    inner: T,
}

impl<T: Rng + 'static> EncodeRng<T> {
    pub fn new(inner: T) -> EncodeRng<T> { EncodeRng { inner: inner } }
}

impl<T: SeedableRng<S> + Rng + 'static, S> SeedableRng<S> for EncodeRng<T> {
    fn reseed(&mut self, seed: S) { self.inner.reseed(seed); }

    fn from_seed(seed: S) -> EncodeRng<T> { EncodeRng::new(SeedableRng::from_seed(seed)) }
}

impl<T: Rng> Rng for EncodeRng<T> {
    fn next_u32(&mut self) -> u32 { self.inner.next_u32() }
}

impl<T: Rng + 'static> serde::Serialize for EncodeRng<T> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut vec = Vec::new();
        unsafe {
            let view = self as *const _ as *const u8;
            for i in 0..(mem::size_of::<T>()) {
                vec.push(*view.offset(i as isize));
            }
        }
        vec.serialize(s)
    }
}

impl<'a, T: Rng + 'static> serde::Deserialize<'a> for EncodeRng<T> {
    fn deserialize<D: serde::Deserializer<'a>>(d: D) -> Result<Self, D::Error> {
        let blob: Vec<u8> = serde::Deserialize::deserialize(d)?;
        unsafe {
            if blob.len() == mem::size_of::<T>() {
                Ok(EncodeRng::new(mem::transmute_copy(&blob[0])))
            } else {
                Err(serde::de::Error::invalid_length(
                    blob.len(),
                    &"Bad inner RNG length",
                ))
            }
        }
    }
}

/// Lazily evaluated random permutation.
pub struct RandomPermutation<'a, R: Rng + 'static> {
    remain: usize,
    shuffle: VecMap<usize>,
    rng: &'a mut R,
}

impl<'a, R: Rng + 'static> RandomPermutation<'a, R> {
    pub fn new(rng: &'a mut R, n: usize) -> RandomPermutation<'a, R> {
        RandomPermutation {
            remain: n,
            shuffle: VecMap::new(),
            rng: rng,
        }
    }
}

impl<'a, R: Rng + 'static> Iterator for RandomPermutation<'a, R> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.remain == 0 {
            return None;
        }

        let swap_idx = self.rng.gen_range(0, self.remain);
        self.remain -= 1;

        let head = *self.shuffle.get(self.remain).unwrap_or(&self.remain);
        Some(self.shuffle.insert(swap_idx, head).unwrap_or(swap_idx))
    }
}

/// Independent sampling trait.
///
/// Implemented as a convenience struct since `rand::distributions::IndependentSample` always wants
/// a boilerplate implementation for `rand::distributions::Sample` as well.
pub trait IndependentSample<Support>: Sized {
    /// Sample a single value from the distribution.
    fn ind_sample<R: Rng>(&self, rng: &mut R) -> Support;

    /// Create an endless iterator sampling values from the distribution.
    fn iter<'a, 'b, R: Rng + 'a>(
        &'b self,
        rng: &'a mut R,
    ) -> SampleIterator<'a, 'b, R, Support, Self> {
        SampleIterator {
            rng,
            sample: self,
            phantom: ::std::marker::PhantomData,
        }
    }
}

pub struct SampleIterator<'a, 'b, R: Rng + 'a, Support, S: IndependentSample<Support> + 'b> {
    rng: &'a mut R,
    sample: &'b S,
    phantom: ::std::marker::PhantomData<Support>,
}

impl<'a, 'b, R: Rng + 'a, Support, S: IndependentSample<Support> + 'b> Iterator
    for SampleIterator<'a, 'b, R, Support, S> {
    type Item = Support;

    fn next(&mut self) -> Option<Self::Item> { Some(self.sample.ind_sample(self.rng)) }
}
