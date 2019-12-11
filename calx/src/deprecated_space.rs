use crate::cell::{CellSpace, CellVector};
use euclid::{vec3, Vector2D, Vector3D};

/// Define a transformation from a space to `CellSpace`.
///
/// Define this trait for your custom space to get `Space` trait automatically derived for you.
///
/// # Defining projections
///
/// First write the X and Y axis vectors of your projected space as column vecttors of a 2x2
/// matrix. For example the `CellSpace` X-axis unit vector becomes (2, 0) and the Y-axis unit
/// vector becomes (-1, 1) in the prefab map `TextSpace`. So we get the matrix
///
/// ```notrust
/// | 2  -1 |
/// | 0   1 |
/// ```
///
/// This is the your unprojection. For the projection matrix, compute the inverse, you'll get
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
/// [v[0] * M[0] + v[1] * M[1], v[0] * M[2] + v[1] * M[3]]
/// ```
pub trait Transformation {
    type Element: Copy;
    // To make this even more automagical, could just make the user specify a 2x2 transformation
    // matrix here, then figure out how to invert the matrix at compile time to keep unprojection
    // efficient and set up conversions between the probably integer element types and
    // the transformation matrix that needs to be floating point. LLVM should be able to optimize
    // away a 2x2 matrix inversion function called with const inputs.

    /// Transform `CellSpace` coordinates to this space.
    fn unproject<V: Into<[i32; 2]>>(v: V) -> [Self::Element; 2];

    /// Transform coordinates of this space to `CellSpace`.
    fn project<V: Into<[Self::Element; 2]>>(v: V) -> [i32; 2];
}

impl Transformation for CellSpace {
    type Element = i32;

    fn unproject<V: Into<[i32; 2]>>(v: V) -> [Self::Element; 2] { v.into() }
    fn project<V: Into<[Self::Element; 2]>>(v: V) -> [i32; 2] { v.into() }
}

/// Projection from a different space into `CellSpace`.
pub trait DeprecatedSpace {
    /// Project from local space to `CellSpace`.
    fn to_cell_space(self) -> CellVector;

    /// Project from `CellSpace` to local space.
    fn from_cell_space(cell: CellVector) -> Self;
}

impl<T, U> DeprecatedSpace for Vector2D<T, U>
where
    T: Copy,
    U: Transformation<Element = T>,
{
    fn to_cell_space(self) -> CellVector { U::project(self).into() }
    fn from_cell_space(cell: CellVector) -> Self { U::unproject(cell).into() }
}

impl<T, U> DeprecatedSpace for Vector3D<T, U>
where
    T: Copy + Default,
    U: Transformation<Element = T>,
{
    fn to_cell_space(self) -> CellVector {
        let v = [self.x, self.y];
        U::project(v).into()
    }
    fn from_cell_space(cell: CellVector) -> Self {
        let v = U::unproject(cell);
        vec3(v[0], v[1], Default::default())
    }
}
