use colors::{SRgba, scolor};
use euclid::{self, point2, TypedPoint2D, TypedRect, TypedVector2D, vec2};
use image::{self, Pixel};
use space::{CellSpace, CellVector, Transformation, Space};
use std::collections::{hash_map, HashMap};
use std::error::Error;
use std::fmt;
use std::i32;
use std::iter::{FromIterator, IntoIterator};

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

    /// Compute the bounding box for the prefab
    ///
    /// This is an O(n) operation, so make sure to cache the result if your prefab is large.
    pub fn bounds(&self) -> TypedRect<i32, CellSpace> {
        type CellPoint = euclid::TypedPoint2D<i32, CellSpace>;
        TypedRect::from_points(
            self.points
                .iter()
                .map(|(&p, _)| p.to_point())
                .collect::<Vec<CellPoint>>()
                .as_slice(),
        )
    }
}

impl<T> FromIterator<(CellVector, T)> for Prefab<T> {
    fn from_iter<I: IntoIterator<Item = (CellVector, T)>>(iter: I) -> Self {
        Prefab { points: FromIterator::from_iter(iter) }
    }
}

impl<T> IntoIterator for Prefab<T> {
    type Item = (CellVector, T);
    type IntoIter = hash_map::IntoIter<CellVector, T>;

    fn into_iter(self) -> Self::IntoIter { self.points.into_iter() }
}

/// Error from parsing data into a `Prefab` value.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PrefabError {
    /// The prefab data is malformed in some way.
    InvalidInput,
    /// The prefab data is missing a (dataformat specific) anchor that points to coordinate origin.
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

// XXX: This is basically just a TryFrom, which isn't stable yet. Though I can derive it for things
// I can't derive TryFrom for, like the templatized GenericImage, because I'm controlling the trait
// here.

/// A trait for types that can be parsed into a map `Prefab`.
pub trait IntoPrefab<T> {
    fn try_into(self) -> Result<Prefab<T>, PrefabError>;
}


// Text prefabs

/// The text map character coordinate space.
pub struct TextSpace;

pub type TextVector = TypedVector2D<i32, TextSpace>;

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

impl fmt::Display for Prefab<char> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let dummy_origin: (&CellVector, &char) = (&vec2(0, 0), &' ');
        // Ensure that the origin cell shows up in the printout even if it's blank in the prefab.
        let append = if self.iter().find(|&(&pos, _)| pos == vec2(0, 0)).is_none() {
            Some(dummy_origin)
        } else {
            None
        };

        // How far left does the textmap go?
        //
        // Subtract 1 from the result so that we can always fit the origin brackets on the leftmost
        // char if need be.
        let min_x = self.iter()
            .chain(append)
            .map(|(&pos, _)| TextVector::from_cell_space(pos).x)
            .min()
            .unwrap_or(0) - 1;

        // Arrange cells in print order.
        let mut sorted: Vec<(TextVector, char)> = self.iter()
            .chain(append)
            .map(|(&pos, &c)| (TextVector::from_cell_space(pos), c))
            .collect();


        sorted.sort_by(|a, b| (a.0.y, a.0.x).cmp(&(b.0.y, b.0.x)));

        if sorted.len() == 0 {
            return Ok(());
        }

        // Printing position.
        let mut print_y = sorted[0].0.y;
        let mut print_x = min_x;

        for &(pos, c) in &sorted {
            while print_y < pos.y {
                writeln!(f, "")?;
                print_x = min_x;
                print_y += 1;
            }

            while print_x < pos.x - 1 {
                write!(f, " ")?;
                print_x += 1;
            }

            debug_assert_eq!(print_y, pos.y);
            if pos == vec2(0, 0) {
                // Write origin markers around origin cell.
                debug_assert_eq!(print_x, pos.x - 1);
                write!(f, "[{}]", c)?;
                print_x += 3;
            } else {
                // Print x should be in pos.x - 1, except in the case right after the origin marker
                // when it's in pos.x.
                debug_assert!(print_x == pos.x - 1 || print_x == pos.x);
                if print_x < pos.x {
                    write!(f, " ")?;
                    print_x += 1;
                }
                write!(f, "{}", c)?;
                print_x += 1;
            }
        }

        Ok(())
    }
}

impl From<Prefab<char>> for String {
    fn from(prefab: Prefab<char>) -> String { format!("{}", prefab) }
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
    fn try_into(self) -> Result<Prefab<SRgba>, PrefabError> {
        // The coordinate space in which the image is in.
        //type LocalVector = TypedVector2D<i32, U>;
        let image = self.image;

        // Completely black pixels are assumed to be non-data.
        fn convert_nonblack<P: image::Pixel<Subpixel = u8>>(p: P) -> Option<SRgba> {
            let (r, g, b, _) = p.channels4();
            if r != 0 && g != 0 && b != 0 {
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
            if convert_nonblack(image.get_pixel(x, 0)).is_some() {
                if anchor_x.is_some() {
                    return Err(PrefabError::MissingAnchor);
                }
                anchor_x = Some(x as i32);
            }
        }

        for y in min_y..(min_y + h) {
            if convert_nonblack(image.get_pixel(0, y)).is_some() {
                if anchor_y.is_some() {
                    return Err(PrefabError::MissingAnchor);
                }
                anchor_y = Some(y as i32);
            }
        }

        // Get the anchor coordinates and project them to cell space.
        let anchor = vec2::<i32, U>(
            anchor_x.ok_or(PrefabError::MissingAnchor)?,
            anchor_y.ok_or(PrefabError::MissingAnchor)?,
        ).to_cell_space();

        let mut points = HashMap::new();

        for y in (min_y + 1)..(min_y + h) {
            for x in (min_x + 1)..(min_x + w) {
                if let Some(c) = convert_nonblack(image.get_pixel(x, y)) {
                    let p = vec2::<i32, U>(x as i32, y as i32).to_cell_space() - anchor;

                    // Insert the color we get when we first hit this point.
                    points.entry(p).or_insert(c);
                }
            }
        }

        Ok(Prefab { points })
    }
}

impl<U> From<Prefab<SRgba>> for ProjectedImage<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>, U>
where
    U: Transformation<Element = i32>,
{
    fn from(prefab: Prefab<SRgba>) -> Self {
        // Bounds from the prefab, these points are in cell space.
        let bounds = prefab.bounds();
        // Ensure that origin is present, needed for anchor display.
        let points: Vec<TypedPoint2D<i32, U>> = [
            bounds.origin,
            bounds.top_right(),
            bounds.bottom_right(),
            bounds.bottom_left(),
            point2(0, 0),
        ].iter()
            .map(|p| TypedVector2D::from_cell_space(p.to_vector()).to_point())
            .collect();

        // Project bounds into space U for the resulting image.
        let bounds = TypedRect::from_points(points.as_slice());
        debug_assert!({
            let origin: TypedVector2D<i32, U> = TypedVector2D::from_cell_space(vec2(0, 0));
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

        for y in 1..bounds.size.height {
            for x in 1..bounds.size.width {
                let prefab_pos = (bounds.origin.to_vector() + vec2(x - 1, y - 1)).to_cell_space();
                // Don't use #000000 as actual data in your prefab because you'll lose it here.
                // (Add the assert to that effect.)
                let c = if let Some(&c) = prefab.get(prefab_pos) {
                    assert!(
                        c != scolor::BLACK,
                        "Prefab contains full black color, converting to image will lose data"
                    );
                    image::Rgba::from_channels(c.r, c.g, c.b, 0xff)
                } else {
                    image::Rgba::from_channels(0, 0, 0, 0xff)
                };
                image.put_pixel(x as u32, y as u32, c);
            }
        }

        ProjectedImage::new(image)
    }
}

impl<I: image::GenericImage<Pixel = P>, P: image::Pixel<Subpixel = u8>> IntoPrefab<SRgba> for I {
    fn try_into(self) -> Result<Prefab<SRgba>, PrefabError> {
        let t: ProjectedImage<I, CellSpace> = ProjectedImage::new(self);
        t.try_into()
    }
}

impl From<Prefab<SRgba>> for image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
    fn from(prefab: Prefab<SRgba>) -> Self {
        let t: ProjectedImage<Self, CellSpace> = From::from(prefab);
        t.image
    }
}

/// The on-screen minimap pixel coordinate space.
pub struct MinimapSpace;

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
