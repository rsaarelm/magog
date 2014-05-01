use world::area::ChartPos;
use cgmath::point::{Point2};

pub struct Transform {
    center: ChartPos,
}

static CENTER_X: f32 = 320.0;
static CENTER_Y: f32 = 180.0;

impl Transform {
    pub fn new(center: ChartPos) -> Transform { Transform { center: center } }

    pub fn to_screen(&self, loc: ChartPos) -> Point2<f32> {
        let x = (loc.x - self.center.x) as f32;
        let y = (loc.y - self.center.y) as f32;
        Point2::new(CENTER_X + 16.0 * x - 16.0 * y, CENTER_Y + 8.0 * x + 8.0 * y)
    }

    pub fn to_chart(&self, pos: &Point2<f32>) -> ChartPos {
        let column = ((pos.x + 8.0 - CENTER_X) / 16.0).floor();
        let row = ((pos.y - CENTER_Y as f32 - column * 8.0) / 16.0).floor();
        ChartPos::new(
            (column + row) as int + self.center.x,
            row as int + self.center.y)
    }
}
