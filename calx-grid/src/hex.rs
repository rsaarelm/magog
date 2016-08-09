use std::slice;
use std::ops::{Add, Sub};
use std::f32::consts::PI;
use std::cmp::max;
use rand::{Rand, Rng};
use num::Integer;
use euclid::Point2D;

/// Hex grid geometry for vectors.
pub trait HexGeom {
    /// Hex distance represented by a vector.
    fn hex_dist(&self) -> i32;
}

impl HexGeom for Point2D<i32> {
    fn hex_dist(&self) -> i32 {
        if self.x.signum() == self.y.signum() {
            max(self.x.abs(), self.y.abs())
        } else {
            self.x.abs() + self.y.abs()
        }
    }
}

/// Hex grid directions.
#[derive(Copy, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Dir6 {
    North = 0,
    NorthEast,
    SouthEast,
    South,
    SouthWest,
    NorthWest,
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
    pub fn from_v2(v: Point2D<i32>) -> Dir6 {
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
    pub fn from_int(i: i32) -> Dir6 {
        DIRS[i.mod_floor(&6) as usize]
    }

    /// Convert a hex dir into the corresponding unit vector.
    pub fn to_v2(&self) -> Point2D<i32> {
        let v = [[-1, -1], [0, -1], [1, 0], [1, 1], [0, 1], [-1, 0]][*self as usize];
        Point2D::new(v[0], v[1])
    }

    /// Iterate through the six hex dirs in the standard order.
    pub fn iter() -> slice::Iter<'static, Dir6> {
        DIRS.iter()
    }
}

impl Add<i32> for Dir6 {
    type Output = Dir6;
    fn add(self, other: i32) -> Dir6 {
        Dir6::from_int(self as i32 + other)
    }
}

impl Sub<i32> for Dir6 {
    type Output = Dir6;
    fn sub(self, other: i32) -> Dir6 {
        Dir6::from_int(self as i32 - other)
    }
}

impl Rand for Dir6 {
    fn rand<R: Rng>(rng: &mut R) -> Dir6 {
        Dir6::from_int(rng.gen_range(0, 6))
    }
}

static DIRS: [Dir6; 6] = [Dir6::North,
                          Dir6::NorthEast,
                          Dir6::SouthEast,
                          Dir6::South,
                          Dir6::SouthWest,
                          Dir6::NorthWest];

/// Hex grid directions with transitional directions.
#[derive(Copy, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Dir12 {
    North = 0,
    NorthNorthEast,
    NorthEast,
    East,
    SouthEast,
    SouthSouthEast,
    South,
    SouthSouthWest,
    SouthWest,
    West,
    NorthWest,
    NorthNorthWest,
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
        assert!(cluster_size > 0);

        // Dir12 in use from here on.
        let center_dir = begin * 2 + (cluster_size - 1);
        let away_dir: u8 = ((center_dir + 6) % 12) as u8;
        assert!(away_dir < 12);

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

            assert!(cluster_end.is_some()); // Must be some if start is some.

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
    use euclid::Point2D;
    use super::Dir6;
    use super::Dir6::*;
    use super::Dir12;

    #[test]
    fn test_dir6() {
        assert_eq!(North, Dir6::from_int(0));
        assert_eq!(NorthWest, Dir6::from_int(-1));
        assert_eq!(NorthWest, Dir6::from_int(5));
        assert_eq!(NorthEast, Dir6::from_int(1));

        assert_eq!(NorthEast, Dir6::from_v2(Point2D::new(20i32, -21i32)));
        assert_eq!(SouthEast, Dir6::from_v2(Point2D::new(20, -10)));
        assert_eq!(North, Dir6::from_v2(Point2D::new(-10, -10)));
        assert_eq!(South, Dir6::from_v2(Point2D::new(1, 1)));

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
            assert_eq!(Dir6::from_int(i + 3),
                       Dir6::from_v2(Point2D::new(-v.x, -v.y)));

            // Test approximation of longer vectors.
            assert_eq!(d, Dir6::from_v2(Point2D::new(v.x * 3, v.y * 3)));
            assert_eq!(d,
                       Dir6::from_v2(Point2D::new(v.x * 3 + v1.x, v.y * 3 + v1.y)));
            assert_eq!(d,
                       Dir6::from_v2(Point2D::new(v.x * 3 + v2.x, v.y * 3 + v2.y)));
        }
    }

    #[test]
    fn test_dir12() {
        assert_eq!(None,
                   Dir12::away_from(&[false, false, false, false, false, false]));
        assert_eq!(None,
                   Dir12::away_from(&[true, true, true, true, true, true]));
        assert_eq!(None,
                   Dir12::away_from(&[false, true, false, false, true, false]));
        assert_eq!(None,
                   Dir12::away_from(&[true, true, false, true, false, false]));
        assert_eq!(None,
                   Dir12::away_from(&[true, false, true, false, true, false]));
        assert_eq!(Some(Dir12::South),
                   Dir12::away_from(&[true, false, false, false, false, false]));
        assert_eq!(Some(Dir12::East),
                   Dir12::away_from(&[true, false, false, true, true, true]));
        assert_eq!(Some(Dir12::SouthSouthWest),
                   Dir12::away_from(&[true, true, false, false, false, false]));
        assert_eq!(Some(Dir12::SouthWest),
                   Dir12::away_from(&[true, true, true, false, false, false]));
        assert_eq!(Some(Dir12::SouthSouthEast),
                   Dir12::away_from(&[true, true, false, false, true, true]));
    }
}
