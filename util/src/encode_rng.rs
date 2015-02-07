use std::mem;
use std::raw;
use rand::{Rng, SeedableRng};
use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};

/// A Rng wrapper that encodes the internal Rng implementation as a binary
/// blob. To be used in games that want to store the current Rng state as a
/// part of the save game.
pub struct EncodeRng<T> {
    inner: T
}

impl<T: Rng+'static> EncodeRng<T> {
    pub fn new(inner: T) -> EncodeRng<T> {
        EncodeRng { inner: inner }
    }
}

impl<T: SeedableRng<S>+Rng+'static, S> SeedableRng<S> for EncodeRng<T> {
    fn reseed(&mut self, seed: S) {
        self.inner.reseed(seed);
    }

    fn from_seed(seed: S) -> EncodeRng<T> {
        EncodeRng::new(SeedableRng::from_seed(seed))
    }
}

impl<T: Rng> Rng for EncodeRng<T> {
    fn next_u32(&mut self) -> u32 { self.inner.next_u32() }
}

impl<T: Rng+'static> Decodable for EncodeRng<T> {
    fn decode<D: Decoder>(d: &mut D) -> Result<EncodeRng<T>, D::Error> {
        let blob: Vec<u8> = try!(Decodable::decode(d));
        unsafe {
            if blob.len() == mem::size_of::<T>() {
                Ok(EncodeRng::new(mem::transmute_copy(&blob[0])))
            } else {
                Err(d.error("Bad RNG blob length"))
            }
        }
    }
}

impl<T: 'static> Encodable for EncodeRng<T> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        let blob: &[u8] = unsafe {
            mem::transmute::<raw::Slice<u8>, &[u8]>(raw::Slice {
                data: mem::transmute(&self.inner),
                len: mem::size_of::<T>(),
            })
        };
        blob.encode(s)
    }
}
