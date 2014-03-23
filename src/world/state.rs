use transform::Transform;
use fov::FovStatus;
use mob::Mob;
use area::{Area, Location};

pub trait State {
    fn transform(&self) -> Transform;
    fn fov(&self, loc: Location) -> FovStatus;
    fn drawable_mob_at<'a>(&'a self, loc: Location) -> Option<&'a Mob>;
    fn area<'a>(&'a self) -> &'a Area;
}

