use euclid::vec2;
use num::Integer;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::cmp::max;
use std::f32::consts::PI;
use std::ops::{Add, Sub};
use std::slice;
use CellVector;

/// Hex grid geometry for vectors.
pub trait HexGeom {
    /// Hex distance represented by a vector.
    fn hex_dist(&self) -> i32;
}

impl HexGeom for CellVector {
    fn hex_dist(&self) -> i32 {
        if self.x.signum() == self.y.signum() {
            max(self.x.abs(), self.y.abs())
        } else {
            self.x.abs() + self.y.abs()
        }
    }
}

// TODO return impl
// No need to muck with custom return iter type then...
/// Return offsets to neighboring hexes.
pub fn hex_neighbors<P, R>(origin: P) -> HexNeighbor<P>
where
    P: Clone + Add<CellVector, Output = R>,
{
    HexNeighbor { origin, i: 0 }
}

pub struct HexNeighbor<P> {
    origin: P,
    i: i32,
}

impl<P, R> Iterator for HexNeighbor<P>
where
    P: Clone + Add<CellVector, Output = R>,
{
    type Item = R;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= 6 {
            None
        } else {
            let ret = Some(self.origin.clone() + Dir6::from_int(self.i).to_v2());
            self.i += 1;
            ret
        }
    }
}

/// Return an iterator for all the points in the hex disc with the given radius.
pub fn hex_disc<P, R>(origin: P, radius: i32) -> HexDisc<P>
where
    P: Clone + Add<CellVector, Output = R>,
{
    HexDisc {
        origin,
        radius,
        i: 0,
        r: 0,
    }
}

pub struct HexDisc<P> {
    origin: P,
    radius: i32,
    i: i32,
    r: i32,
}

impl<P, R> Iterator for HexDisc<P>
where
    P: Clone + Add<CellVector, Output = R>,
{
    type Item = R;

    fn next(&mut self) -> Option<Self::Item> {
        if self.r > self.radius {
            return None;
        }

        if self.r == 0 {
            self.r += 1;
            self.i = 0;
            return Some(self.origin.clone() + vec2(0, 0));
        }

        let sector = self.i / self.r;
        let offset = self.i % self.r;
        let rod = Dir6::from_int(sector);
        let tangent = Dir6::from_int(sector + 2);

        let ret = rod.to_v2() * self.r + tangent.to_v2() * offset;

        self.i += 1;
        if self.i >= 6 * self.r {
            self.i = 0;
            self.r += 1;
        }

        Some(self.origin.clone() + ret)
    }
}

/// Hex grid directions.
#[derive(Copy, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Dir6 {
    North = 0,
    Northeast,
    Southeast,
    South,
    Southwest,
    Northwest,
}

impl Dir6 {
    /// Convert a vector into the closest hex direction.
    ///
    /// ```notrust
    ///        *0*       *1*
    ///           \ 14 15 | 00 01
    ///           13\     |      02
    ///               \   |
    ///         12      \ |        03
    ///     *5* ----------O-X------- *2*
    ///         11        Y \      04
    ///                   |   \
    ///           10      |     \05
    ///             09 08 | 07 06 \
    ///                  *4*       *3*
    ///
    /// The hexadecants (00 to 15) and the hex
    /// directions (*0* to *5*) around the origin.
    /// ```
    ///
    /// Vectors that are in a space between two hex direction vectors are
    /// rounded to a hexadecant, then assigned the hex direction whose vector
    /// is nearest to that hexadecant.
    pub fn from_v2(v: CellVector) -> Dir6 {
        let hexadecant = {
            let width = PI / 8.0;
            let mut radian = (v.x as f32).atan2(-v.y as f32);
            if radian < 0.0 {
                radian += 2.0 * PI
            }
            (radian / width).floor() as i32
        };

        Dir6::from_int(match hexadecant {
            13 | 14 => 0,
            15 | 0 | 1 => 1,
            2 | 3 | 4 => 2,
            5 | 6 => 3,
            7 | 8 | 9 => 4,
            10 | 11 | 12 => 5,
            _ => panic!("Bad hexadecant"),
        })
    }

    /// Convert an integer to a hex dir using modular arithmetic.
    pub fn from_int(i: i32) -> Dir6 { DIRS[i.mod_floor(&6) as usize] }

    /// Convert a hex dir into the corresponding unit vector.
    pub fn to_v2(&self) -> CellVector { CellVector::from(*self) }

    /// Iterate through the six hex dirs in the standard order.
    pub fn iter() -> slice::Iter<'static, Dir6> { DIRS.iter() }
}

impl Add<i32> for Dir6 {
    type Output = Dir6;
    fn add(self, other: i32) -> Dir6 { Dir6::from_int(self as i32 + other) }
}

impl Sub<i32> for Dir6 {
    type Output = Dir6;
    fn sub(self, other: i32) -> Dir6 { Dir6::from_int(self as i32 - other) }
}

impl Distribution<Dir6> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Dir6 { Dir6::from_int(rng.gen_range(0, 6)) }
}

impl From<Dir6> for CellVector {
    fn from(d: Dir6) -> Self {
        const DIRS: [(i32, i32); 6] = [(-1, -1), (0, -1), (1, 0), (1, 1), (0, 1), (-1, 0)];

        let (x, y) = DIRS[d as usize];
        vec2(x, y)
    }
}

static DIRS: [Dir6; 6] = [
    Dir6::North,
    Dir6::Northeast,
    Dir6::Southeast,
    Dir6::South,
    Dir6::Southwest,
    Dir6::Northwest,
];

/// Hex grid directions with transitional directions.
#[derive(Copy, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Dir12 {
    North = 0,
    NorthNortheast,
    Northeast,
    East,
    Southeast,
    SouthSoutheast,
    South,
    SouthSouthwest,
    Southwest,
    West,
    Northwest,
    NorthNorthwest,
}

impl Dir12 {
    /// If there is exactly one cluster of neighbors in the neighbor mask,
    /// return a direction pointing away from that cluster.
    pub fn away_from(neighbors: &[bool; 6]) -> Option<Dir12> {
        use std::mem;

        let (begin, end) = match find_cluster(neighbors) {
            Some((a, b)) => (a, b),
            None => return None,
        };

        if !is_single_cluster(neighbors, begin, end) {
            return None;
        }

        let cluster_size = if end < begin {
            end + 6 - begin
        } else {
            end - begin
        };
        debug_assert!(cluster_size > 0);

        // Dir12 in use from here on.
        let center_dir = begin * 2 + (cluster_size - 1);
        let away_dir: u8 = ((center_dir + 6) % 12) as u8;
        debug_assert!(away_dir < 12);

        // XXX: Unsafe because I'm too lazy to do int conversion func by hand.
        return Some(unsafe { mem::transmute(away_dir) });

        fn find_cluster(neighbors: &[bool; 6]) -> Option<(usize, usize)> {
            // Start of the active cluster, inclusive.
            let mut cluster_start = None;
            // End of the active cluster, exclusive.
            let mut cluster_end = None;

            for i in 0..6 {
                if cluster_start.is_none() && neighbors[i] && !neighbors[(i + 5) % 6] {
                    cluster_start = Some(i);
                }

                if cluster_end.is_none() && !neighbors[i] && neighbors[(i + 5) % 6] {
                    cluster_end = Some(i);
                }
            }

            if cluster_start.is_none() {
                return None;
            }

            debug_assert!(cluster_end.is_some()); // Must be some if start is some.

            Some((cluster_start.unwrap(), cluster_end.unwrap()))
        }

        fn is_single_cluster(neighbors: &[bool; 6], start: usize, end: usize) -> bool {
            let mut in_cluster = true;

            for i in 0..6 {
                if (start + i) % 6 == end {
                    in_cluster = false;
                }

                if neighbors[(start + i) % 6] != in_cluster {
                    return false;
                }
            }

            true
        }
    }
}

#[cfg(test)]
mod test {
    use super::hex_disc;
    use super::Dir12;
    use super::Dir6;
    use super::Dir6::*;
    use euclid::vec2;

    #[test]
    fn test_dir6() {
        assert_eq!(North, Dir6::from_int(0));
        assert_eq!(Northwest, Dir6::from_int(-1));
        assert_eq!(Northwest, Dir6::from_int(5));
        assert_eq!(Northeast, Dir6::from_int(1));

        assert_eq!(Northeast, Dir6::from_v2(vec2(20i32, -21i32)));
        assert_eq!(Southeast, Dir6::from_v2(vec2(20, -10)));
        assert_eq!(North, Dir6::from_v2(vec2(-10, -10)));
        assert_eq!(South, Dir6::from_v2(vec2(1, 1)));

        for i in 0..6 {
            let d = Dir6::from_int(i);
            let v = d.to_v2();
            let v1 = Dir6::from_int(i - 1).to_v2();
            let v2 = Dir6::from_int(i + 1).to_v2();

            // Test static iter
            assert_eq!(Some(d), Dir6::iter().nth(i as usize).map(|&x| x));

            // Test vector mapping.
            assert_eq!(d, Dir6::from_v2(v));

            // Test opposite dir vector mapping.
            assert_eq!(Dir6::from_int(i + 3), Dir6::from_v2(vec2(-v.x, -v.y)));

            // Test approximation of longer vectors.
            assert_eq!(d, Dir6::from_v2(vec2(v.x * 3, v.y * 3)));
            assert_eq!(d, Dir6::from_v2(vec2(v.x * 3 + v1.x, v.y * 3 + v1.y)));
            assert_eq!(d, Dir6::from_v2(vec2(v.x * 3 + v2.x, v.y * 3 + v2.y)));
        }
    }

    #[test]
    fn test_dir12() {
        assert_eq!(
            None,
            Dir12::away_from(&[false, false, false, false, false, false])
        );
        assert_eq!(
            None,
            Dir12::away_from(&[true, true, true, true, true, true])
        );
        assert_eq!(
            None,
            Dir12::away_from(&[false, true, false, false, true, false])
        );
        assert_eq!(
            None,
            Dir12::away_from(&[true, true, false, true, false, false])
        );
        assert_eq!(
            None,
            Dir12::away_from(&[true, false, true, false, true, false])
        );
        assert_eq!(
            Some(Dir12::South),
            Dir12::away_from(&[true, false, false, false, false, false])
        );
        assert_eq!(
            Some(Dir12::East),
            Dir12::away_from(&[true, false, false, true, true, true])
        );
        assert_eq!(
            Some(Dir12::SouthSouthwest),
            Dir12::away_from(&[true, true, false, false, false, false])
        );
        assert_eq!(
            Some(Dir12::Southwest),
            Dir12::away_from(&[true, true, true, false, false, false])
        );
        assert_eq!(
            Some(Dir12::SouthSoutheast),
            Dir12::away_from(&[true, true, false, false, true, true])
        );
    }

    #[test]
    fn test_hex_disc() {
        use super::HexGeom;
        use euclid::vec2;

        for y in -8i32..8 {
            for x in -8i32..8 {
                let vec = vec2(x, y);

                for r in 0..8 {
                    assert_eq!(
                        vec.hex_dist() <= r,
                        hex_disc(vec2(0, 0), r).find(|&v| vec == v).is_some()
                    );
                }
            }
        }
    }
}
