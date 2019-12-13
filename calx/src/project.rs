use crate::space::Space;
use euclid::{Point2D, Vector2D};

/// Transformation between different geometries, eg. tile map and on-screen isometric grid.
///
/// # Defining projections
///
/// First write the X and Y axis vectors of your projected space as column vectors of a 2x2 matrix.
/// For example the `CellSpace` X-axis unit vector becomes (2, 0) and the Y-axis unit vector
/// becomes (-1, 1) in the prefab map `TextSpace`. So we get the matrix
///
/// ```notrust
/// | 2  -1 |
/// | 0   1 |
/// ```
///
/// This is the projection from `CellSpace` to `TextSpace`. For the one from `TextSpace` to
/// `CellSpace`, compute the inverse and you'll get
///
/// ```notrust
/// | 2  -1 | ^-1     | 1/2  1/2 |
/// | 0   1 |      =  |   0    1 |
/// ```
///
/// The projection formula for vector v and projection matrix M is Mv, ie (with row-major matrix
/// representation):
///
/// ```notrust
/// vec2(vec.x * M[0] + vec.y * M[1], vec.x * M[2] + vec.y * M[3])
/// ```
///
/// Projections from a fine space to a coarse space often use a handwritten custom projection
/// method rather than just a straightforward matrix multiplication, since they may want to
/// describe different tiling shapes (eg. hexagons) than the square grid that matrix multiplication
/// will give you.
///
/// ```
/// use calx::{project, CellSpace, CellVector, ProjectVec, Space};
/// use euclid::{vec2, Vector2D};
///
/// // Project cell grid into isometric on-screen graphics.
/// struct IsometricSpace;
/// impl Space for IsometricSpace {
///     type T = i32;
/// }
/// type IsometricVector = Vector2D<i32, IsometricSpace>;
///
/// // Isometric tiles are 32x16 lozenges, so we get vectors x = [16, 8], y = [-16, 8].
///
/// // | 16  -16 | ^-1     1/ |  1  2 |
/// // |  8    8 |      =  32 | -1  2 |
///
/// impl project::From<CellSpace> for IsometricSpace {
///     fn vec_from(
///         vec: Vector2D<<CellSpace as Space>::T, CellSpace>,
///     ) -> Vector2D<Self::T, Self> {
///         vec2(vec.x * 16 - vec.y * 16, vec.x * 8 + vec.y * 8)
///     }
/// }
///
/// impl project::From<IsometricSpace> for CellSpace {
///     fn vec_from(
///         vec: Vector2D<<IsometricSpace as Space>::T, IsometricSpace>,
///     ) -> Vector2D<Self::T, Self> {
///         let (x, y) = (vec.x as f32, vec.y as f32);
///         let (x, y) = ((x + y * 2.0) / 32.0, (-x + y * 2.0) / 32.0);
///         vec2(x.round() as i32, y.round() as i32)
///     }
/// }
///
/// assert_eq!(
///     vec2(16, 8),
///     CellVector::new(1, 0).project::<IsometricSpace>()
/// );
/// assert_eq!(
///     vec2(0, 16),
///     CellVector::new(1, 1).project::<IsometricSpace>()
/// );
/// assert_eq!(
///     vec2(6, 2),
///     IsometricVector::new(64, 64).project::<CellSpace>()
/// );
/// ```
pub trait From<U: Space>: Sized + Space {
    fn vec_from(vec: Vector2D<U::T, U>) -> Vector2D<Self::T, Self>;

    fn point_from(point: Point2D<U::T, U>) -> Point2D<Self::T, Self> {
        Self::vec_from(point.to_vector()).to_point()
    }
}

// Identity projection
impl<U: Space> From<U> for U {
    fn vec_from(vec: Vector2D<U::T, U>) -> Vector2D<Self::T, Self> { vec }
}
