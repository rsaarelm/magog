use cgmath::point::{Point2};
use area::Area;
use fov::Fov;

pub struct AreaView<'s> {
    priv a: &'s Area,
    seen: ~Fov,
    remembered: ~Fov,
    pos: Point2<int>,
}

impl<'s> AreaView<'s> {
    pub fn new(a: &'s Area) -> AreaView<'s> {
        AreaView {
            a: a,
            seen: ~Fov::new(),
            remembered: ~Fov::new(),
            pos: Point2::new(0, 0),
        }
    }
}
