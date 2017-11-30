use euclid::{self, vec2};
use space::{CellVector, Transformation, Space};
use std::collections::{hash_map, HashMap};
use std::error::Error;
use std::fmt;
use std::i32;

/// The text map character coordinate space.
struct TextSpace;

pub type TextVector = euclid::TypedVector2D<i32, TextSpace>;

// |  2 0 |
// | -1 1 |
//
// | 1/2 0 |
// | 1/2 1 |

impl Transformation for TextSpace {
    type Element = i32;

    fn project<V: Into<[Self::Element; 2]>>(v: V) -> [i32; 2] {
        let v = v.into();
        [(v[0] + v[1]) / 2, v[1]]
    }

    fn unproject<V: Into<[i32; 2]>>(v: V) -> [Self::Element; 2] {
        let v = v.into();
        [2 * v[0] - v[1], v[1]]
    }
}

impl TextSpace {
    /// Which of the two possible map lattices is this vector in?
    pub fn in_even_lattice(v: TextVector) -> bool { (v.x + v.y) % 2 == 0 }
}


/// The on-screen minimap pixel coordinate space.
struct MinimapSpace;

// |  2  1 |
// | -2  1 |
//
// | 1/4  -1/4 |
// | 1/2   1/2 |

impl Transformation for MinimapSpace {
    type Element = i32;

    fn project<V: Into<[Self::Element; 2]>>(v: V) -> [i32; 2] {
        let v = v.into();
        [v[0] / 4 + v[1] / 2, v[1] / 2 - v[0] / 4]
    }

    fn unproject<V: Into<[i32; 2]>>(v: V) -> [Self::Element; 2] {
        let v = v.into();
        [2 * v[0] - 2 * v[1], v[0] + v[1]]
    }
}


// Idea was there'd be both 'Floating' and 'Anchored' prefabs, but I don't think I actually have an
// use case for the 'Floating' one. So just using the 'Anchored' one and calling it a generic
// Prefab.

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PrefabError {
    InvalidInput,
    MissingAnchor,
}

impl fmt::Display for PrefabError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.description()) }
}

impl Error for PrefabError {
    fn description(&self) -> &str {
        match self {
            &PrefabError::InvalidInput => "Invalid input",
            &PrefabError::MissingAnchor => "Anchor not found in input",
        }
    }
}

// TODO: Once `std::convert::TryFrom` becomes stable, use that instead of this custom trait.

/// A trait for types that can be parsed into a map `Prefab`.
pub trait IntoPrefab<T> {
    fn try_into(self) -> Result<Prefab<T>, PrefabError>;
}

/// A piece of 2D map data with a fixed origin position.
///
/// # Examples
///
/// ```
/// # extern crate euclid;
/// # extern crate calx;
/// # fn main() {
///
/// use euclid::vec2;
/// use calx::{Prefab, IntoPrefab};
///
/// let string_map = Prefab::parse(r#"
///   1 2
///  3[4]5
///   6 7"#).expect("Failed to parse string map");
///
/// for &(c, p) in &[
///   ('1', (-1, -1)),
///   ('2', ( 0, -1)),
///   ('3', (-1,  0)),
///   ('4', ( 0,  0)),
///   ('5', ( 1,  0)),
///   ('6', ( 0,  1)),
///   ('7', ( 1,  1))] {
///     assert_eq!(Some(&c), string_map.get(vec2(p.0, p.1)));
/// }
///
/// # }
/// ```
#[derive(Clone)]
pub struct Prefab<T> {
    points: HashMap<CellVector, T>,
}

impl<T> Prefab<T> {
    pub fn parse<U: IntoPrefab<T>>(value: U) -> Result<Prefab<T>, PrefabError> { value.try_into() }

    pub fn get(&self, pos: CellVector) -> Option<&T> { self.points.get(&pos) }

    pub fn iter(&self) -> hash_map::Iter<CellVector, T> { self.points.iter() }
}

// Not using FromStr even when the input type is String, since there's a conversion family here
// where the sources can be strings or images and I want the interface to be uniform for both.

impl<S: Into<String>> IntoPrefab<char> for S {
    fn try_into(self) -> Result<Prefab<char>, PrefabError> {
        /// Recognize the point set that only contains the origin markup and return the
        /// corresponding origin value.
        fn is_origin(points: &HashMap<TextVector, char>) -> Option<TextVector> {
            if points.len() != 2 {
                return None;
            }
            let mut it = points.iter();
            let (mut p1, mut p2) = (it.next().unwrap(), it.next().unwrap());

            if p1.0.x > p2.0.x {
                ::std::mem::swap(&mut p1, &mut p2);
            }

            if *p2.0 - *p1.0 == vec2(2, 0) && *p1.1 == '[' && *p2.1 == ']' {
                Some(*p1.0 + vec2(1, 0))
            } else {
                None
            }
        }

        let value: String = self.into();

        let mut odds = HashMap::new();
        let mut evens = HashMap::new();
        for (y, line) in value.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                if !c.is_whitespace() {
                    let vec = vec2(x as i32, y as i32);
                    if TextSpace::in_even_lattice(vec) {
                        evens.insert(vec, c);
                    } else {
                        odds.insert(vec, c);
                    }
                }
            }
        }

        let (offset, data) = if let Some(offset) = is_origin(&odds) {
            (offset, &evens)
        } else if let Some(offset) = is_origin(&evens) {
            (offset, &odds)
        } else {
            return Err(PrefabError::InvalidInput);
        };

        let mut points = HashMap::new();
        for text_data in data.iter() {
            // Set origin
            let text_pos = *text_data.0 - offset;
            // Store into cell space.
            points.insert(text_pos.to_cell_space(), *text_data.1);
        }

        Ok(Prefab { points })
    }
}

impl From<Prefab<char>> for String {
    fn from(prefab: Prefab<char>) -> String {
        // How far left does the textmap go?
        //
        // Subtract 1 from the result so that we can always fit the origin brackets on the leftmost
        // char if need be.
        let min_x = prefab
            .iter()
            .map(|(&pos, _)| TextVector::from_cell_space(pos).x)
            .min()
            .unwrap_or(0) - 1;

        unimplemented!();
    }
}
