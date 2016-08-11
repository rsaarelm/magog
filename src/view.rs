use std::iter::Map;
use euclid::{Point2D, Rect};
use backend;
use world::{ScreenChart, World};

/// Useful general constant for cell dimension ops.
pub static PIXEL_UNIT: i32 = 16;

/// Transform from chart space (unit is one map cell) to view space (unit is
/// one pixel).
pub fn chart_to_view(chart_pos: Point2D<i32>) -> Point2D<f32> {
    Point2D::new((chart_pos.x * PIXEL_UNIT - chart_pos.y * PIXEL_UNIT) as f32,
                 (chart_pos.x * PIXEL_UNIT / 2 + chart_pos.y * PIXEL_UNIT / 2) as f32)
}

/// Transform from view space (unit is one pixel) to chart space (unit is one
/// map cell).
pub fn view_to_chart(view_pos: Point2D<f32>) -> Point2D<i32> {
    let c = PIXEL_UNIT as f32 / 2.0;
    let column = ((view_pos.x + c) / (c * 2.0)).floor();
    let row = ((view_pos.y - column * c) / (c * 2.0)).floor();
    Point2D::new((column + row) as i32, row as i32)
}

/// Return the chart positions for which chart_to_view is inside view_rect.
pub fn cells_in_view_rect(view_rect: Rect<f32>)
                          -> Map<ColumnRectIter, fn(Point2D<i32>) -> Point2D<i32>> {
    let p0 = pixel_to_min_column(view_rect.origin);
    let p1 = pixel_to_max_column(view_rect.bottom_right());
    ColumnRectIter {
        x: p0.x,
        y: p0.y,
        upper_row: p0.x % 2 == 0,
        x0: p0.x,
        x1: p1.x,
        y1: p1.y,
    }
    .map(column_to_chart)
}

/// Transform to the column space point that contains the pixel space point
/// when looking for minimum column space point. (The column space rows
/// overlap, so minimum and maximum points may differ.)
fn pixel_to_min_column(pixel_pos: Point2D<f32>) -> Point2D<i32> {
    Point2D::new((pixel_pos.x as i32 - PIXEL_UNIT) / PIXEL_UNIT,
                 (pixel_pos.y as i32 - PIXEL_UNIT * 2) / PIXEL_UNIT)
}

/// Transform to the column space point that contains the pixel space point
/// when looking for maximum column space point. (The column space rows
/// overlap, so minimum and maximum points may differ.)
fn pixel_to_max_column(pixel_pos: Point2D<f32>) -> Point2D<i32> {
    Point2D::new((pixel_pos.x as i32 + PIXEL_UNIT) / PIXEL_UNIT,
                 (pixel_pos.y as i32 + PIXEL_UNIT * 2) / PIXEL_UNIT)
}

/// Transform a column space point to a chart space point.
fn column_to_chart(cr: Point2D<i32>) -> Point2D<i32> {
    Point2D::new(((1i32 + cr.x + 2i32 * cr.y) as f32 / 2f32).floor() as i32,
                 (-(cr.x - 1) as f32 / 2f32).floor() as i32 + cr.y)
}

/// Draw a view space described by the chart.
pub fn draw_world(context: &mut backend::Context,
                  world: &World,
                  chart: &ScreenChart,
                  screen_rect: &Rect<f32>,
                  screen_offset: &Point2D<f32>) {
    unimplemented!();
}

#[derive(Copy, Clone)]
pub struct ColumnRectIter {
    x: i32,
    y: i32,
    // To prevent ordering artifacts, a hex column layout iterator needs to
    // return each row in two parts, first the upper row of hexes offsetted
    // up, then the lower row.
    upper_row: bool,
    x0: i32,
    x1: i32,
    y1: i32,
}

impl Iterator for ColumnRectIter {
    type Item = Point2D<i32>;
    fn next(&mut self) -> Option<Point2D<i32>> {
        if self.y >= self.y1 {
            return None;
        }
        let ret = Some(Point2D::new(self.x, self.y));
        self.x = self.x + 2;

        if self.x >= self.x1 {
            self.x = self.x0 +
                     if ((self.x0 % 2) == 1) ^ !self.upper_row {
                1
            } else {
                0
            };
            if self.upper_row {
                self.upper_row = false;
            } else {
                self.y = self.y + 1;
                self.upper_row = true;
            }
        }
        ret
    }
}

#[cfg(test)]
mod test {
    use euclid::Point2D;
    use super::column_to_chart;

    #[test]
    fn c2c() {
        assert_eq!(Point2D::new(-1, 0), column_to_chart(Point2D::new(-1, -1)));
        assert_eq!(Point2D::new(-1, -1), column_to_chart(Point2D::new(0, -1)));
        assert_eq!(Point2D::new(0, -1), column_to_chart(Point2D::new(1, -1)));

        assert_eq!(Point2D::new(0, 1), column_to_chart(Point2D::new(-1, 0)));
        assert_eq!(Point2D::new(0, 0), column_to_chart(Point2D::new(0, 0)));
        assert_eq!(Point2D::new(1, 0), column_to_chart(Point2D::new(1, 0)));

        assert_eq!(Point2D::new(1, 2), column_to_chart(Point2D::new(-1, 1)));
        assert_eq!(Point2D::new(1, 1), column_to_chart(Point2D::new(0, 1)));
        assert_eq!(Point2D::new(2, 1), column_to_chart(Point2D::new(1, 1)));

        assert_eq!(Point2D::new(-3, -1), column_to_chart(Point2D::new(-2, -2)));
        assert_eq!(Point2D::new(1, 3), column_to_chart(Point2D::new(-2, 2)));
        assert_eq!(Point2D::new(3, 1), column_to_chart(Point2D::new(2, 2)));
        assert_eq!(Point2D::new(-1, -3), column_to_chart(Point2D::new(2, -2)));
    }
}
