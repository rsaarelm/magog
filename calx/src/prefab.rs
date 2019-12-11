use crate::{
    alg_misc::bounding_rect,
    cell::{CellSpace, CellVector},
    deprecated_space::{DeprecatedSpace, Transformation},
};
use euclid::{point2, vec2, Point2D, Vector2D};
use image::{self, Pixel};
use num::Integer;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::i32;
use std::iter::{FromIterator, IntoIterator};
use vitral::{scolor, SRgba};

/// Error from parsing data into a `Prefab` value.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PrefabError {
    /// The prefab data is malformed in some way.
    InvalidInput,
    /// The prefab data is missing a (dataformat specific) anchor that points to coordinate origin.
    MissingAnchor,
    /// The prefab data contains multiple anchors.
    MultipleAnchors,
}

impl fmt::Display for PrefabError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Error for PrefabError {
    fn description(&self) -> &str {
        match *self {
            PrefabError::InvalidInput => "Invalid input",
            PrefabError::MissingAnchor => "Anchor not found in input",
            PrefabError::MultipleAnchors => {
                "Multiple anchor positions found in input"
            }
        }
    }
}

/// A trait for types that can be parsed into a map `Prefab`.
///
/// # Examples
///
/// ```
/// # fn main() {
///
/// use std::collections::HashMap;
/// use euclid::vec2;
/// use calx::{CellVector, IntoPrefab};
///
/// let map: HashMap<CellVector, char> = r#"
///   1 2
///  3[4]5
///   6 7"#.into_prefab().expect("Failed to parse string map");
///
/// for &(c, p) in &[
///   ('1', (-1, -1)),
///   ('2', ( 0, -1)),
///   ('3', (-1,  0)),
///   ('4', ( 0,  0)),
///   ('5', ( 1,  0)),
///   ('6', ( 0,  1)),
///   ('7', ( 1,  1))] {
///     assert_eq!(Some(&c), map.get(&vec2(p.0, p.1)));
/// }
///
/// # }
/// ```
pub trait IntoPrefab<T> {
    fn into_prefab<P: FromIterator<(CellVector, T)>>(
        self,
    ) -> Result<P, PrefabError>;
}

/// Trait for types that can be constructed from prefab data.
///
/// # Examples
///
/// ```
/// # fn main() {
///
/// use std::collections::HashMap;
/// use euclid::vec2;
/// use calx::{CellVector, FromPrefab};
///
/// let mut prefab: HashMap<CellVector, char> = HashMap::new();
/// prefab.insert(vec2(1, 0), 'x');
/// prefab.insert(vec2(0, 1), 'y');
///
/// assert_eq!(" [ ]x\n y", &String::from_prefab(&prefab));
/// # }
/// ```
pub trait FromPrefab {
    type Cell;
    // XXX Would be nice to be able to have something more generic as the parameter, but the
    // implementations need both random access and iteration over all values and there isn't really
    // a good idiom for that.
    fn from_prefab(prefab: &HashMap<CellVector, Self::Cell>) -> Self;
}

// Text prefabs

/// The oblique projection text map character coordinate space.
pub struct TextSpace;

pub type TextVector = Vector2D<i32, TextSpace>;

// | 2  -1 |
// | 0   1 |
//
// | 1/2  1/2 |
// |   0    1 |

impl Transformation for TextSpace {
    type Element = i32;

    fn unproject<V: Into<[i32; 2]>>(v: V) -> [Self::Element; 2] {
        let v = v.into();
        [2 * v[0] - v[1], v[1]]
    }

    fn project<V: Into<[Self::Element; 2]>>(v: V) -> [i32; 2] {
        let v = v.into();
        [(v[0] + v[1]) / 2, v[1]]
    }
}

impl TextSpace {
    /// Which of the two possible map lattices is this vector in?
    pub fn in_even_lattice(v: TextVector) -> bool { (v.x + v.y) % 2 == 0 }
}

impl<S: Into<String>> IntoPrefab<char> for S {
    fn into_prefab<P: FromIterator<(CellVector, char)>>(
        self,
    ) -> Result<P, PrefabError> {
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

        Ok(P::from_iter(data.iter().map(|(p, c)| {
            // Set origin
            let text_pos = *p - offset;
            // Store into cell space.
            (text_pos.to_cell_space(), *c)
        })))
    }
}

impl FromPrefab for String {
    type Cell = char;

    fn from_prefab(prefab: &HashMap<CellVector, Self::Cell>) -> Self {
        use std::fmt::Write;

        let mut ret = String::new();

        let dummy_origin: (&CellVector, &char) = (&vec2(0, 0), &' ');
        // Ensure that the origin cell shows up in the printout even if it's blank in the prefab.
        let append = if !prefab.contains_key(&vec2(0, 0)) {
            Some(dummy_origin)
        } else {
            None
        };

        // How far left does the textmap go?
        //
        // Subtract 1 from the result so that we can always fit the origin brackets on the leftmost
        // char if need be.
        let min_x = prefab
            .iter()
            .chain(append)
            .map(|(&pos, _)| TextVector::from_cell_space(pos).x)
            .min()
            .unwrap_or(0)
            - 1;

        // Arrange cells in print order.
        let mut sorted: Vec<(TextVector, char)> = prefab
            .iter()
            .chain(append)
            .map(|(&pos, &c)| (TextVector::from_cell_space(pos), c))
            .collect();

        sorted.sort_by(|a, b| (a.0.y, a.0.x).cmp(&(b.0.y, b.0.x)));

        if sorted.is_empty() {
            return "".to_string();
        }

        // Printing position.
        let mut print_y = sorted[0].0.y;
        let mut print_x = min_x;

        for &(pos, c) in &sorted {
            while print_y < pos.y {
                let _ = writeln!(ret);
                print_x = min_x;
                print_y += 1;
            }

            while print_x < pos.x - 1 {
                let _ = write!(ret, " ");
                print_x += 1;
            }

            debug_assert_eq!(print_y, pos.y);
            if pos == vec2(0, 0) {
                // Write origin markers around origin cell.
                debug_assert_eq!(print_x, pos.x - 1);
                let _ = write!(ret, "[{}]", c);
                print_x += 3;
            } else {
                // Print x should be in pos.x - 1, except in the case right after the origin marker
                // when it's in pos.x.
                debug_assert!(print_x == pos.x - 1 || print_x == pos.x);
                if print_x < pos.x {
                    let _ = write!(ret, " ");
                    print_x += 1;
                }
                let _ = write!(ret, "{}", c);
                print_x += 1;
            }
        }

        ret
    }
}

/// Wrapper to signify that a text prefab has no spaces between cells.
///
/// A dense text map has no way to specify the origin position, so the origin is just placed at the
/// point corresponding to the top left corner of the bounding box of the prefab. Use the sparse
/// map style if you want to specify an origin for your prefab.
///
/// # Examples
///
/// ```
/// # fn main() {
/// use std::collections::HashMap;
/// use euclid::vec2;
/// use calx::{CellVector, IntoPrefab, DenseTextMap};
///
/// let map: HashMap<CellVector, char> = DenseTextMap(r#"
///   12
///   34"#).into_prefab().expect("Failed to parse string map");
///
/// for &(c, p) in &[
///   ('1', (0, 0)),
///   ('2', (1, 0)),
///   ('3', (0, 1)),
///   ('4', (1, 1))] {
///     assert_eq!(Some(&c), map.get(&vec2(p.0, p.1)));
/// }
///
/// # }
/// ```
pub struct DenseTextMap<'a>(pub &'a str);

impl<'a> IntoPrefab<char> for DenseTextMap<'a> {
    fn into_prefab<P: FromIterator<(CellVector, char)>>(
        self,
    ) -> Result<P, PrefabError> {
        let mut elts = Vec::new();
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        for (y, line) in self.0.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                if !c.is_whitespace() {
                    let (x, y) = (x as i32, y as i32);
                    elts.push((vec2(x, y), c));
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                }
            }
        }

        Ok(P::from_iter(
            elts.into_iter().map(|(p, c)| (p - vec2(min_x, min_y), c)),
        ))
    }
}

// Image prefabs

/// Wrapper for image maps coupled with a projection.
///
/// NB: The image prefab converter ignores alpha channel and treats full black (#000000) as empty
/// space. Do not use the full black color in your color prefab data, it will get lost in
/// conversion.
pub struct ProjectedImage<I, U> {
    pub image: I,
    unit_type: ::std::marker::PhantomData<U>,
}

impl<I, P, U> ProjectedImage<I, U>
where
    I: image::GenericImage<Pixel = P>,
    P: image::Pixel<Subpixel = u8>,
    U: Transformation<Element = i32>,
{
    pub fn new(image: I) -> ProjectedImage<I, U> {
        ProjectedImage {
            image,
            unit_type: ::std::marker::PhantomData,
        }
    }
}

impl<I, P, U> IntoPrefab<SRgba> for ProjectedImage<I, U>
where
    I: image::GenericImage<Pixel = P>,
    P: image::Pixel<Subpixel = u8>,
    U: Transformation<Element = i32>,
{
    fn into_prefab<Q: FromIterator<(CellVector, SRgba)>>(
        self,
    ) -> Result<Q, PrefabError> {
        // The coordinate space in which the image is in.
        //type LocalVector = Vector2D<i32, U>;
        let image = self.image;

        // Completely black pixels are assumed to be non-data.
        fn convert_nonblack<P: image::Pixel<Subpixel = u8>>(
            p: P,
        ) -> Option<SRgba> {
            let (r, g, b, _) = p.channels4();
            if r != 0 || g != 0 || b != 0 {
                Some(SRgba::new(r, g, b, 0xff))
            } else {
                None
            }
        }

        let (min_x, min_y, w, h) = image.bounds();
        let mut anchor_x = None;
        let mut anchor_y = None;

        // The top and left lines of the image must be used for anchor. They need to contain
        // exactly one non-black pixel that points the origin coordinate.
        for x in min_x..(min_x + w) {
            if convert_nonblack(image.get_pixel(x, min_y)).is_some() {
                if anchor_x.is_some() {
                    return Err(PrefabError::MultipleAnchors);
                }
                anchor_x = Some(x as i32);
            }
        }

        for y in min_y..(min_y + h) {
            if convert_nonblack(image.get_pixel(min_x, y)).is_some() {
                if anchor_y.is_some() {
                    return Err(PrefabError::MultipleAnchors);
                }
                anchor_y = Some(y as i32);
            }
        }

        // Get the anchor coordinates and project them to cell space.
        let anchor = vec2::<i32, U>(
            anchor_x.ok_or(PrefabError::MissingAnchor)?,
            anchor_y.ok_or(PrefabError::MissingAnchor)?,
        );

        let mut seen_cells = HashSet::new();

        // XXX: Traversing rectangle points was too annoying to type in an iterator expression,
        // just caching the points in a Vec here instead.
        let mut points = Vec::new();
        for y in (min_y + 1)..(min_y + h) {
            for x in (min_x + 1)..(min_x + w) {
                points.push((x, y));
            }
        }

        Ok(Q::from_iter(points.into_iter().flat_map(|(x, y)| {
            if let Some(c) = convert_nonblack(image.get_pixel(x, y)) {
                let p =
                    vec2::<i32, U>(x as i32 - anchor.x, y as i32 - anchor.y)
                        .to_cell_space();

                // Only insert a cell the first time we see it.
                if !seen_cells.contains(&p) {
                    seen_cells.insert(p);
                    Some((p, c))
                } else {
                    None
                }
            } else {
                None
            }
        })))
    }
}

impl<I: image::GenericImage<Pixel = P>, P: image::Pixel<Subpixel = u8>>
    IntoPrefab<SRgba> for I
{
    fn into_prefab<Q: FromIterator<(CellVector, SRgba)>>(
        self,
    ) -> Result<Q, PrefabError> {
        let t: ProjectedImage<I, CellSpace> = ProjectedImage::new(self);
        t.into_prefab()
    }
}

impl<U> FromPrefab
    for ProjectedImage<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, U>
where
    U: Transformation<Element = i32>,
{
    type Cell = SRgba;

    fn from_prefab(prefab: &HashMap<CellVector, Self::Cell>) -> Self {
        // Project points from the prefab (plus origin which we need in the image frame for anchor
        // encoding) to calculate the projected bounds.
        let points: Vec<Point2D<i32, U>> = prefab
            .iter()
            .map(|(&p, _)| Vector2D::from_cell_space(p).to_point())
            .chain(Some(point2(0, 0)))
            .collect();

        // Project bounds into space U for the resulting image.
        let bounds = bounding_rect(points.as_slice());

        debug_assert!({
            let origin: Vector2D<i32, U> =
                Vector2D::from_cell_space(vec2(0, 0));
            bounds.origin.x <= origin.x && bounds.origin.y <= origin.y
        });

        // Add space for the origin axes
        let mut image = image::ImageBuffer::new(
            bounds.size.width as u32 + 1,
            bounds.size.height as u32 + 1,
        );

        // Draw anchor dots.
        image.put_pixel(
            (1 - bounds.origin.x) as u32,
            0,
            image::Rgba::from_channels(0xff, 0xff, 0, 0xff),
        );
        image.put_pixel(
            0,
            (1 - bounds.origin.y) as u32,
            image::Rgba::from_channels(0xff, 0xff, 0, 0xff),
        );

        for y in 0..bounds.size.height {
            for x in 0..bounds.size.width {
                let prefab_pos =
                    (bounds.origin.to_vector() + vec2(x, y)).to_cell_space();
                // Don't use #000000 as actual data in your prefab because you'll lose it here.
                // (Add the assert to that effect.)
                let c = if let Some(&c) = prefab.get(&prefab_pos) {
                    assert!(
                        c != scolor::BLACK,
                        "Prefab contains full black color, converting to image will lose data"
                    );
                    image::Rgba::from_channels(c.r, c.g, c.b, 0xff)
                } else {
                    image::Rgba::from_channels(0, 0, 0, 0xff)
                };
                image.put_pixel(x as u32 + 1, y as u32 + 1, c);
            }
        }

        ProjectedImage::new(image)
    }
}

impl FromPrefab for image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
    type Cell = SRgba;

    fn from_prefab(prefab: &HashMap<CellVector, Self::Cell>) -> Self {
        let t: ProjectedImage<Self, CellSpace> =
            FromPrefab::from_prefab(prefab);
        t.image
    }
}

/// The on-screen minimap pixel coordinate space.
pub struct MinimapSpace;

// | 2  -2 |
// | 1   1 |
//
// |  1/4  1/2 |
// | -1/4  1/2 |

impl Transformation for MinimapSpace {
    type Element = i32;

    fn unproject<V: Into<[i32; 2]>>(v: V) -> [Self::Element; 2] {
        let v = v.into();
        [2 * v[0] - 2 * v[1], v[0] + v[1]]
    }

    fn project<V: Into<[Self::Element; 2]>>(v: V) -> [i32; 2] {
        let mut v = v.into();
        // Snap in square cells
        v[0] &= -1; // Two-pixel columns
        if v[0].mod_floor(&4) < 2 {
            // Even column
            v[1] &= !1;
        } else {
            // Odd column
            v[1] = ((v[1] + 1) & !1) - 1;
        }

        let v = [v[0] as f32, v[1] as f32];
        [
            (v[0] / 4.0 + v[1] / 2.0).round() as i32,
            (v[1] / 2.0 - v[0] / 4.0).round() as i32,
        ]
    }
}

#[cfg(test)]
mod test {
    use super::MinimapSpace;
    use crate::deprecated_space::Transformation;

    #[test]
    fn test_minimap_projection() {
        assert_eq!([0, 0], MinimapSpace::project([0, 0]));
        assert_eq!([0, 0], MinimapSpace::project([1, 0]));
        assert_eq!([0, 0], MinimapSpace::project([0, 1]));
        assert_eq!([0, 0], MinimapSpace::project([1, 1]));
    }
}
