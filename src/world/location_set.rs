use std::collections::HashMap;
use std::mem;
use super::location::Location;

/// Compact Location set collection
#[derive(Eq, PartialEq, Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct LocationSet {
    /// Chunks of 8x8 locations. The key has top 5 bits of the x and y
    /// coordinates of the location catenated into one integer for the
    /// location of the chunk, and the value uses the 64 bits of the u64 to
    /// cover the 8x8 chunk with a bitmap.
    chunks: HashMap<u32, u64>
}

impl LocationSet {
    pub fn new() -> LocationSet {
        LocationSet {
            chunks: HashMap::new()
        }
    }

    /// Return the chunk index and the bit offset for a location.
    #[inline]
    fn chunk(loc: &Location) -> (u32, u64) {
        let ux: u8 = unsafe { mem::transmute(loc.x) };
        let uy: u8 = unsafe { mem::transmute(loc.y) };

        let index = (ux as u32 >> 3) + ((uy as u32 >> 3) << 5);
        let bit = (ux % 8) as u64 + ((uy % 8) << 3) as u64;

        (index, 1 << bit)
    }

    pub fn contains(&self, loc: &Location) -> bool {
        let (index, bit) = LocationSet::chunk(loc);
        match self.chunks.get(&index) {
            Some(b) if b & bit != 0 => { true }
            _ => { false }
        }
    }

    pub fn clear(&mut self) {
        self.chunks.clear();
    }

    pub fn insert(&mut self, loc: Location) {
        let (index, bit) = LocationSet::chunk(&loc);
        let n = bit | self.chunks.get(&index).unwrap_or(&0);
        self.chunks.insert(index, n);
    }

    pub fn _remove(&mut self, loc: &Location) {
        let (index, bit) = LocationSet::chunk(loc);
        let n = !bit & self.chunks.get(&index).unwrap_or(&0);
        if n == 0 {
            self.chunks.remove(&index);
        } else {
            self.chunks.insert(index, n);
        }
    }

    pub fn extend<I: Iterator<Item=Location>>(&mut self, iter: I) {
        for i in iter {
            self.insert(i);
        }
    }
}
