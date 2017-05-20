use location::Location;
use std::collections::HashMap;

/// Compact Location set collection
#[derive(Eq, PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
pub struct LocationSet {
    /// Chunks of 8x8 locations.
    ///
    /// The locations are assigned to the chunks using their Morton codes (Z-order curves). The low
    /// 6 bits in the Morton code assign the position in the chunk and the higher bits give the
    /// chunk index. Sequences of 6 low bits in 2D Morton coding correspond to 8x8 squares on the
    /// map grid.
    chunks: HashMap<u32, u64>,
}

impl LocationSet {
    /// Return the chunk index and the bit offset for a location.
    #[inline]
    fn chunk(loc: &Location) -> (u32, u64) {
        let morton = loc.to_morton();
        (morton >> 6, 1 << (morton % 64))
    }

    pub fn contains(&self, loc: &Location) -> bool {
        let (index, bit) = LocationSet::chunk(loc);
        match self.chunks.get(&index) {
            Some(b) => b & bit != 0,
            _ => false,
        }
    }

    pub fn clear(&mut self) { self.chunks.clear(); }

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

    pub fn extend<I: Iterator<Item = Location>>(&mut self, iter: I) {
        for i in iter {
            self.insert(i);
        }
    }
}
