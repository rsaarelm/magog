use std::collections::HashMap;
use std::iter::{FromIterator};
use euclid::{Point2D, Rect};
use calx_grid::{FovValue, HexFov};
use world::{Location, World};
use world::query;

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


#[derive(Clone)]
struct ScreenFov<'a> {
    w: &'a World,
    screen_area: Rect<f32>,
    origins: Vec<Location>,
}

impl<'a> PartialEq for ScreenFov<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.w as *const World == other.w as *const World &&
        self.screen_area == other.screen_area && self.origins == other.origins
    }
}

impl<'a> Eq for ScreenFov<'a> {}

impl<'a> FovValue for ScreenFov<'a> {
    fn advance(&self, offset: Point2D<i32>) -> Option<Self> {
        if !self.screen_area.contains(&chart_to_view(offset)) {
            return None;
        }

        let loc = self.origins[0] + offset;

        let mut ret = self.clone();
        // Go through a portal if terrain on our side of the portal is a void cell.
        //
        // With non-void terrain on top of the portal, just show our side and stay on the current
        // frame as far as FOV is concerned.
        if let Some(dest) = query::visible_portal(self.w, loc) {
            ret.origins.insert(0, dest - offset);
        }

        Some(ret)
    }
}

/// Return the field of view chart for drawing a screen.
pub fn screen_fov(w: &World,
                  origin: Location,
                  screen_area: Rect<f32>)
                  -> HashMap<Point2D<i32>, Vec<Location>> {
    let init = ScreenFov {
        w: w,
        screen_area: screen_area,
        origins: vec![origin],
    };

    HashMap::from_iter(HexFov::new(init).map(|(pos, a)| (pos, a.origins)))
}
