use crate::{
    alg_misc::bounding_rect,
    cell::{CellSpace, CellVector},
    project,
    space::{ProjectVec, Space},
};
use euclid::{point2, vec2, Point2D, Rect, Vector2D};
use image::Pixel;
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.to_string()) }
}

impl Error for PrefabError {
    fn description(&self) -> &str {
        match *self {
            PrefabError::InvalidInput => "Invalid input",
            PrefabError::MissingAnchor => "Anchor not found in input",
            PrefabError::MultipleAnchors => "Multiple anchor positions found in input",
        }
    }
}

/// A trait for types that can be parsed into a map `Prefab`.
///
/// # Examples
///
/// ```
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
/// ```
pub trait IntoPrefab<T> {
    fn into_prefab<P: FromIterator<(CellVector, T)>>(self) -> Result<P, PrefabError>;
}

/// Trait for types that can be constructed from prefab data.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use euclid::vec2;
/// use calx::{CellVector, FromPrefab};
///
/// let mut prefab: HashMap<CellVector, char> = HashMap::new();
/// prefab.insert(vec2(1, 0), 'x');
/// prefab.insert(vec2(0, 1), 'y');
///
/// assert_eq!(" [ ]x\n y", &String::from_prefab(&prefab));
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

impl Space for TextSpace {
    type T = i32;
}

// | 1/2  1/2 |
// |   0    1 |

impl project::From<TextSpace> for CellSpace {
    fn vec_from(vec: Vector2D<<TextSpace as Space>::T, TextSpace>) -> Vector2D<Self::T, Self> {
        vec2((vec.x + vec.y) / 2, vec.y)
    }
}

// | 2  -1 |
// | 0   1 |

impl project::From<CellSpace> for TextSpace {
    fn vec_from(vec: Vector2D<<CellSpace as Space>::T, CellSpace>) -> Vector2D<Self::T, Self> {
        vec2(2 * vec.x - vec.y, vec.y)
    }
}

pub type TextVector = Vector2D<i32, TextSpace>;

impl TextSpace {
    /// Which of the two possible map lattices is this vector in?
    pub fn in_even_lattice(v: TextVector) -> bool { (v.x + v.y) % 2 == 0 }
}

impl<S: Into<String>> IntoPrefab<char> for S {
    fn into_prefab<P: FromIterator<(CellVector, char)>>(self) -> Result<P, PrefabError> {
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
            (text_pos.project(), *c)
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
            .map(|(&pos, _)| pos.project::<TextSpace>().x)
            .min()
            .unwrap_or(0)
            - 1;

        // Arrange cells in print order.
        let mut sorted: Vec<(TextVector, char)> = prefab
            .iter()
            .chain(append)
            .map(|(&pos, &c)| (pos.project(), c))
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
/// use std::collections::HashMap;
/// use euclid::vec2;
/// use calx::{CellVector, IntoPrefab, FromPrefab, DenseTextMap};
///
/// let map: HashMap<CellVector, char> = DenseTextMap("
///   12
///   34").into_prefab().expect("Failed to parse string map");
///
/// for &(c, p) in &[
///   ('1', (0, 0)),
///   ('2', (1, 0)),
///   ('3', (0, 1)),
///   ('4', (1, 1))] {
///     assert_eq!(Some(&c), map.get(&vec2(p.0, p.1)));
///
/// let new_map = DenseTextMap::from_prefab(&map);
/// assert_eq!(new_map.0, "\
/// 12
/// 34");
/// }
/// ```
pub struct DenseTextMap<S>(pub S);

impl<S: AsRef<str>> IntoPrefab<char> for DenseTextMap<S> {
    fn into_prefab<P: FromIterator<(CellVector, char)>>(self) -> Result<P, PrefabError> {
        let mut elts = Vec::new();
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        for (y, line) in self.0.as_ref().lines().enumerate() {
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

impl FromPrefab for DenseTextMap<String> {
    type Cell = char;

    fn from_prefab(prefab: &HashMap<CellVector, Self::Cell>) -> Self {
        let bounds = Rect::from_points(prefab.keys().map(|v| v.to_point()));
        let mut ret = String::new();
        for y in bounds.min_y()..(bounds.max_y() + 1) {
            for x in bounds.min_x()..(bounds.max_x() + 1) {
                if let Some(&c) = prefab.get(&vec2(x, y)) {
                    ret.push(c);
                } else {
                    ret.push(' ');
                }
            }
            ret.truncate(ret.trim_end().len());
            ret.push_str("\n");
        }
        ret.truncate(ret.trim_end().len());

        DenseTextMap(ret)
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
    U: Space<T = i32>,
    CellSpace: project::From<U>,
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
    U: Space<T = i32>,
    CellSpace: project::From<U>,
{
    fn into_prefab<Q: FromIterator<(CellVector, SRgba)>>(self) -> Result<Q, PrefabError> {
        // The coordinate space in which the image is in.
        //type LocalVector = Vector2D<i32, U>;
        let image = self.image;

        // Completely black pixels are assumed to be non-data.
        fn convert_nonblack<P: image::Pixel<Subpixel = u8>>(p: P) -> Option<SRgba> {
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
                let p = vec2::<U::T, U>(x as i32 - anchor.x, y as i32 - anchor.y).project();

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

impl<I: image::GenericImage<Pixel = P>, P: image::Pixel<Subpixel = u8>> IntoPrefab<SRgba> for I {
    fn into_prefab<Q: FromIterator<(CellVector, SRgba)>>(self) -> Result<Q, PrefabError> {
        let t: ProjectedImage<I, CellSpace> = ProjectedImage::new(self);
        t.into_prefab()
    }
}

impl<U> FromPrefab for ProjectedImage<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, U>
where
    U: Space<T = i32>,
    CellSpace: project::From<U>,
    U: project::From<CellSpace>,
{
    type Cell = SRgba;

    fn from_prefab(prefab: &HashMap<CellVector, Self::Cell>) -> Self {
        // Project points from the prefab (plus origin which we need in the image frame for anchor
        // encoding) to calculate the projected bounds.
        let points: Vec<Point2D<i32, U>> = prefab
            .iter()
            .map(|(&p, _)| p.project().to_point())
            .chain(Some(point2(0, 0)))
            .collect();

        // Project bounds into space U for the resulting image.
        let bounds = bounding_rect(points.as_slice());

        debug_assert!({
            let origin: Vector2D<U::T, U> = CellVector::new(0, 0).project();
            bounds.origin.x <= origin.x && bounds.origin.y <= origin.y
        });

        // Add space for the origin axes
        let mut image =
            image::ImageBuffer::new(bounds.size.width as u32 + 1, bounds.size.height as u32 + 1);

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
                let prefab_pos: CellVector = (bounds.origin.to_vector() + vec2(x, y)).project();
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
        let t: ProjectedImage<Self, CellSpace> = FromPrefab::from_prefab(prefab);
        t.image
    }
}

/// The on-screen minimap pixel coordinate space.
pub struct MinimapSpace;

impl Space for MinimapSpace {
    type T = i32;
}

// |  1/4  1/2 |
// | -1/4  1/2 |
//
// | 2  -2 |
// | 1   1 |

impl project::From<MinimapSpace> for CellSpace {
    fn vec_from(
        mut vec: Vector2D<<MinimapSpace as Space>::T, MinimapSpace>,
    ) -> Vector2D<Self::T, Self> {
        // Snap in square cells
        vec.x &= -1; // Two-pixel columns
        if vec.x.rem_euclid(4) < 2 {
            // Even column
            vec.y &= !1;
        } else {
            // Odd column
            vec.y = ((vec.y + 1) & !1) - 1;
        }

        let (x, y) = (vec.x as f32, vec.y as f32);
        vec2(
            (x / 4.0 + y / 2.0).round() as i32,
            (y / 2.0 - x / 4.0).round() as i32,
        )
    }
}

impl project::From<CellSpace> for MinimapSpace {
    fn vec_from(vec: Vector2D<<CellSpace as Space>::T, CellSpace>) -> Vector2D<Self::T, Self> {
        vec2(2 * vec.x - 2 * vec.y, vec.x + vec.y)
    }
}

#[cfg(test)]
mod test {
    use super::MinimapSpace;
    use crate::space::ProjectVec;
    use crate::CellSpace;

    #[test]
    fn test_minimap_projection() {
        use euclid::vec2;
        type MinimapVector = euclid::Vector2D<i32, MinimapSpace>;

        assert_eq!(vec2(0, 0), MinimapVector::new(0, 0).project::<CellSpace>());
        assert_eq!(vec2(0, 0), MinimapVector::new(1, 0).project::<CellSpace>());
        assert_eq!(vec2(0, 0), MinimapVector::new(0, 1).project::<CellSpace>());
        assert_eq!(vec2(0, 0), MinimapVector::new(1, 1).project::<CellSpace>());
    }
}
