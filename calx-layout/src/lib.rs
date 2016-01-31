extern crate cgmath;
extern crate num;

mod rect;

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

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {}
}
