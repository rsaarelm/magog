use std::iter::Map;
use calx::{V2, Rect, RectIter};

pub static SCREEN_W: int = 640;
pub static SCREEN_H: int = 360;

/// Useful general constant for cell dimension ops.
static PIXEL_UNIT: int = 16;

/// Draw layer for floor tiles.
pub static FLOOR_Z: f32 = 0.500f32;
/// Draw layer for wall and object tiles.
pub static BLOCK_Z: f32 = 0.400f32;

/// Transform from chart space (unit is one map cell) to view space (unit is
/// one pixel).
pub fn chart_to_view(chart_pos: V2<int>) -> V2<int> {
    V2(chart_pos.0 * PIXEL_UNIT - chart_pos.1 * PIXEL_UNIT,
       chart_pos.0 * PIXEL_UNIT / 2 + chart_pos.1 * PIXEL_UNIT / 2)
}

/// Transform from view space (unit is one pixel) to chart space (unit is one
/// map cell).
pub fn view_to_chart(view_pos: V2<int>) -> V2<int> {
    let c = PIXEL_UNIT as f32 / 2.0;
    let column = ((view_pos.0 as f32 + c) / (c * 2.0)).floor();
    let row = ((view_pos.1 as f32 - column * c) / (c * 2.0)).floor();
    V2((column + row) as int, row as int)
}

/// Return the chart positions for which chart_to_view is inside view_rect.
pub fn cells_in_view_rect<'a>(view_rect: Rect<int>) -> Map<'a, V2<int>, V2<int>, RectIter<int>> {
    let p1 = pixel_to_min_column(view_rect.mn());
    let p2 = pixel_to_max_column(view_rect.mx());
    Rect(p1, p2 - p1)
        .iter().map(|rc| column_to_chart(rc))
}

pub fn cells_on_screen<'a>() -> Map<'a, V2<int>, V2<int>, RectIter<int>> {
    cells_in_view_rect(Rect(V2(-SCREEN_W / 2, -SCREEN_H / 2), V2(SCREEN_W, SCREEN_H)))
}

/// Transform to the column space point that contains the pixel space point
/// when looking for minimum column space point. (The column space rows
/// overlap, so minimum and maximum points differ.)
fn pixel_to_min_column(pixel_pos: V2<int>) -> V2<int> {
    V2((pixel_pos.0 - PIXEL_UNIT) / PIXEL_UNIT,
       (pixel_pos.1 - PIXEL_UNIT * 2) / PIXEL_UNIT)
}

/// Transform to the column space point that contains the pixel space point
/// when looking for maximum column space point. (The column space rows
/// overlap, so minimum and maximum points differ.)
fn pixel_to_max_column(pixel_pos: V2<int>) -> V2<int> {
    V2((pixel_pos.0 + PIXEL_UNIT) / PIXEL_UNIT,
       (pixel_pos.1 + PIXEL_UNIT) / PIXEL_UNIT)
}

/// Transform a column space point to a chart space point.
fn column_to_chart(cr: V2<int>) -> V2<int> {
    V2(((1 + cr.0 + 2 * cr.1) as f32 / 2f32).floor() as int,
       (-(cr.0 - 1) as f32 / 2f32).floor() as int + cr.1)
}

#[cfg(test)]
mod test {
    use calx::V2;
    use super::column_to_chart;

    #[test]
    fn c2c() {
        assert_eq!(V2(-1,  0), column_to_chart(V2(-1, -1)));
        assert_eq!(V2(-1, -1), column_to_chart(V2( 0, -1)));
        assert_eq!(V2( 0, -1), column_to_chart(V2( 1, -1)));

        assert_eq!(V2( 0,  1), column_to_chart(V2(-1,  0)));
        assert_eq!(V2( 0,  0), column_to_chart(V2( 0,  0)));
        assert_eq!(V2( 1,  0), column_to_chart(V2( 1,  0)));

        assert_eq!(V2( 1,  2), column_to_chart(V2(-1,  1)));
        assert_eq!(V2( 1,  1), column_to_chart(V2( 0,  1)));
        assert_eq!(V2( 2,  1), column_to_chart(V2( 1,  1)));

        assert_eq!(V2(-3, -1), column_to_chart(V2(-2, -2)));
        assert_eq!(V2( 1,  3), column_to_chart(V2(-2,  2)));
        assert_eq!(V2( 3,  1), column_to_chart(V2( 2,  2)));
        assert_eq!(V2(-1, -3), column_to_chart(V2( 2, -2)));
    }
}
