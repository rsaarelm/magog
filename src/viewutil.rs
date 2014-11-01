use calx::{V2, Rect};

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
pub fn cells_in_view_rect(view_rect: Rect<int>) -> Vec<V2<int>> {
    // Add CELL_W to bottom so that the tops of lower tiles will get drawn.
    let (min, max) = (view_rect.mn(), view_rect.mx() + V2(0, CELL_W));
    let mut cells = Rect(view_to_chart(min), V2(0, 0));
    cells.grow(view_to_chart(min + V2((view_rect.1).0, 0)));
    cells.grow(view_to_chart(min + V2(0, (view_rect.1).1 + CELL_W)));
    cells.grow(view_to_chart(max));

    // XXX: Adds some points that are outside the view rectangle.
    let mut ret = vec![];
    for y in range((cells.0).1, cells.mx().1 + 1) {
        for x in range((cells.0).0, cells.mx().0 + 1) {
            ret.push(V2(x, y));
        }
    }

    ret
}

pub fn cells_on_screen() -> Vec<V2<int>> {
    cells_in_view_rect(Rect(V2(-SCREEN_W / 2, -SCREEN_H / 2), V2(SCREEN_W, SCREEN_H)))
}
