use std::mem;
use vec_map::VecMap;
use rand::{Rng, SeedableRng};
use serde;
use to_log_odds;

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

    fn with_chance(&mut self, p: f32) -> bool { self.gen_range(0.0, 1.0) < p }

    fn log_odds(&mut self) -> f32 { to_log_odds(self.gen_range(0.0, 1.0)) }

    fn with_log_odds(&mut self, db: f32) -> bool { db > self.log_odds() }
}

/// A wrapper that makes a Rng implementation encodable.
///
/// For games that want to store the current Rng state as a part of the save
/// game. Works by casting the Rng representation into a binary blob, will
/// crash and burn if the Rng struct is not plain-old-data.
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

impl<T: Rng + 'static> serde::Deserialize for EncodeRng<T> {
    fn deserialize<D: serde::Deserializer>(d: D) -> Result<Self, D::Error> {
        let blob: Vec<u8> = serde::Deserialize::deserialize(d)?;
        unsafe {
            if blob.len() == mem::size_of::<T>() {
                Ok(EncodeRng::new(mem::transmute_copy(&blob[0])))
            } else {
                Err(serde::de::Error::invalid_length(blob.len(), &"Bad inner RNG length"))
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
        let ret = Some(self.shuffle.insert(swap_idx, head).unwrap_or(swap_idx));
        ret
    }
}
