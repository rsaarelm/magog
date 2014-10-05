use calx::{V2, Rect};

pub static SCREEN_W: int = 640;
pub static SCREEN_H: int = 360;

/// Visual terrain cell width.
pub static CELL_W: int = 32;

/// Transform from chart space (unit is one map cell) to view space (unit is
/// one pixel).
pub fn chart_to_view(chart_pos: V2<int>) -> V2<int> {
    V2(chart_pos.0 * CELL_W / 2 - chart_pos.1 * CELL_W / 2,
       chart_pos.0 * CELL_W / 4 + chart_pos.1 * CELL_W / 4)
}

/// Transform from view space (unit is one pixel) to chart space (unit is one
/// map cell).
pub fn view_to_chart(view_pos: V2<int>) -> V2<int> {
    let c = CELL_W as f32 / 4.0;
    let column = ((view_pos.0 as f32 + c) / (c * 2.0)).floor();
    let row = ((view_pos.1 as f32 - column * c) / (c * 2.0)).floor();
    V2((column + row) as int, row as int)
}

/// Return the chart positions for which chart_to_view is inside view_rect.
pub fn cells_in_view_rect(view_rect: Rect<int>) -> Vec<V2<int>> {
    unimplemented!();
}
