extern crate cgmath;
extern crate num;

mod projection;
mod rect;

pub use projection::Projection;
pub use rect::Rect;

/// Rectangle anchoring points.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Anchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Top,
    Left,
    Right,
    Bottom,
    Center,
}
