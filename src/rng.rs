use rand::{seq, Rng, SeedableRng, XorShiftRng};
use std::hash::Hash;
use vec_map::VecMap;
use Deciban;

/// Seed a RNG from any hashable value.
pub fn seeded_rng(seed: &impl Hash) -> XorShiftRng {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;

    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    let hash = hasher.finish().to_be();
    // XorShift seed mustn't be all-0.
    let hash = if hash == 0 { 1 } else { hash };

    let seed = unsafe { ::std::mem::transmute::<[u64; 2], [u8; 16]>([hash, hash]) };
    SeedableRng::from_seed(seed)
}

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

    /// Pick a single random element from a slice.
    fn pick_slice_ref<'a, U>(&mut self, slice: &'a [U]) -> Option<&'a U>;

    /// Pick a single random element from a slice.
    fn pick_slice<'a, U: Clone>(&mut self, slice: &'a [U]) -> Option<U>;

    /// Pick a single random element from an iterator
    fn pick_iter<U>(&mut self, iter: impl IntoIterator<Item = U>) -> Option<U>;
}

impl<T: Rng + ?Sized> RngExt for T {
    fn coinflip(&mut self) -> bool { self.gen_bool(1.0 / 2.0) }

    fn one_chance_in(&mut self, n: u32) -> bool { self.gen_bool(1.0 / n as f64) }

    fn with_chance(&mut self, p: f32) -> bool { self.gen_range(0.0, 1.0) < p }

    fn with_log_odds(&mut self, db: Deciban) -> bool { db > self.gen::<Deciban>() }

    fn pick_slice_ref<'a, U>(&mut self, slice: &'a [U]) -> Option<&'a U> {
        seq::sample_slice_ref(self, slice, 1).pop()
    }

    fn pick_slice<U: Clone>(&mut self, slice: &[U]) -> Option<U> {
        seq::sample_slice(self, slice, 1).pop()
    }

    /// Pick a single random element from an iterator
    fn pick_iter<U>(&mut self, iter: impl IntoIterator<Item = U>) -> Option<U> {
        seq::sample_iter(self, iter, 1).unwrap_or(Vec::new()).pop()
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
            rng,
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
