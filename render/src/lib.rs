extern crate cgmath;
extern crate calx_color;
extern crate calx_layout;
extern crate calx_grid;
extern crate content;

use std::iter::Map;
use calx_layout::{Rect, Anchor};
use cgmath::{Vector2, vec2};

mod render_terrain;

pub use render_terrain::{Angle, render_terrain};

// TODO: Turn these into parameters.
static SCREEN_W: u32 = 640;
static SCREEN_H: u32 = 360;

/// Useful general constant for cell dimension ops.
pub static PIXEL_UNIT: i32 = 16;

/// Draw layer for floor tiles.
pub static FLOOR_Z: f32 = 0.312f32;
/// Draw layer for wall and object tiles.
pub static BLOCK_Z: f32 = 0.311f32;

/// Draw layer for visual effects
pub static FX_Z: f32 = 0.300f32;

/// Transform from chart space (unit is one map cell) to view space (unit is
/// one pixel).
pub fn chart_to_view(chart_pos: Vector2<i32>) -> Vector2<i32> {
    vec2(chart_pos.x * PIXEL_UNIT - chart_pos.y * PIXEL_UNIT,
         chart_pos.x * PIXEL_UNIT / 2 + chart_pos.y * PIXEL_UNIT / 2)
}

/// Transform from chart space into the default on-screen space centered on
/// window center.
pub fn chart_to_screen(chart_pos: Vector2<i32>) -> Vector2<f32> {
    (chart_to_view(chart_pos) + vec2(SCREEN_W as i32 / 2, SCREEN_H as i32 / 2))
        .cast()
}

/// Convert depth difference to pixel offset.
pub fn level_z_to_view(z: i32) -> Vector2<i32> {
    vec2(0, z * -PIXEL_UNIT)
}

/// Transform from view space (unit is one pixel) to chart space (unit is one
/// map cell).
pub fn view_to_chart(view_pos: Vector2<i32>) -> Vector2<i32> {
    let c = PIXEL_UNIT as f32 / 2.0;
    let column = ((view_pos.x as f32 + c) / (c * 2.0)).floor();
    let row = ((view_pos.y as f32 - column * c) / (c * 2.0)).floor();
    vec2((column + row) as i32, row as i32)
}

/// Return the chart positions for which chart_to_view is inside view_rect.
pub fn cells_in_view_rect(view_rect: Rect<i32>)
                          -> Map<ColumnRectIter, fn(Vector2<i32>) -> Vector2<i32>> {
    let p0 = pixel_to_min_column(Vector2::from(view_rect.point(Anchor::TopLeft)));
    let p1 = pixel_to_max_column(Vector2::from(view_rect.point(Anchor::BottomRight)));
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

pub fn cells_on_screen() -> Map<ColumnRectIter, fn(Vector2<i32>) -> Vector2<i32>> {
    cells_in_view_rect(Rect::new_ts(vec2(-(SCREEN_W as i32) / 2, -(SCREEN_H as i32) / 2),
                            vec2((SCREEN_W as i32), SCREEN_H as i32)))
}

/// Transform to the column space point that contains the pixel space point
/// when looking for minimum column space point. (The column space rows
/// overlap, so minimum and maximum points may differ.)
fn pixel_to_min_column(pixel_pos: Vector2<i32>) -> Vector2<i32> {
    vec2((pixel_pos.x - PIXEL_UNIT) / PIXEL_UNIT,
         (pixel_pos.y - PIXEL_UNIT * 2) / PIXEL_UNIT)
}

/// Transform to the column space point that contains the pixel space point
/// when looking for maximum column space point. (The column space rows
/// overlap, so minimum and maximum points may differ.)
fn pixel_to_max_column(pixel_pos: Vector2<i32>) -> Vector2<i32> {
    vec2((pixel_pos.x + PIXEL_UNIT) / PIXEL_UNIT,
         (pixel_pos.y + PIXEL_UNIT * 2) / PIXEL_UNIT)
}

/// Transform a column space point to a chart space point.
fn column_to_chart(cr: Vector2<i32>) -> Vector2<i32> {
    vec2(((1i32 + cr.x + 2i32 * cr.y) as f32 / 2f32).floor() as i32,
         (-(cr.x - 1) as f32 / 2f32).floor() as i32 + cr.y)
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
    type Item = Vector2<i32>;
    fn next(&mut self) -> Option<Vector2<i32>> {
        if self.y >= self.y1 {
            return None;
        }
        let ret = Some(vec2(self.x, self.y));
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
    use cgmath::{Vector, vec2};
    use super::column_to_chart;

    #[test]
    fn c2c() {
        assert_eq!(vec2(-1, 0), column_to_chart(vec2(-1, -1)));
        assert_eq!(vec2(-1, -1), column_to_chart(vec2(0, -1)));
        assert_eq!(vec2(0, -1), column_to_chart(vec2(1, -1)));

        assert_eq!(vec2(0, 1), column_to_chart(vec2(-1, 0)));
        assert_eq!(vec2(0, 0), column_to_chart(vec2(0, 0)));
        assert_eq!(vec2(1, 0), column_to_chart(vec2(1, 0)));

        assert_eq!(vec2(1, 2), column_to_chart(vec2(-1, 1)));
        assert_eq!(vec2(1, 1), column_to_chart(vec2(0, 1)));
        assert_eq!(vec2(2, 1), column_to_chart(vec2(1, 1)));

        assert_eq!(vec2(-3, -1), column_to_chart(vec2(-2, -2)));
        assert_eq!(vec2(1, 3), column_to_chart(vec2(-2, 2)));
        assert_eq!(vec2(3, 1), column_to_chart(vec2(2, 2)));
        assert_eq!(vec2(-1, -3), column_to_chart(vec2(2, -2)));
    }
}
