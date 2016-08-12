use std::collections::{HashMap, HashSet};
use euclid::Point2D;
use std::iter::FromIterator;
use calx_grid::{HexFov, HexGeom};
use world::World;
use query;
use location::Location;

// TODO: Bake stopping predicate into this?
fn screen_fov_advance<F>(w: &World,
                         is_valid_pos: &F,
                         offset: Point2D<i32>,
                         origins: &Vec<Location>)
                         -> Option<Vec<Location>>
    where F: Fn(Point2D<i32>) -> bool
{
    if !is_valid_pos(offset) {
        return None;
    }

    let loc = origins[0] + offset;

    let mut ret = origins.clone();
    // Go through a portal if terrain on our side of the portal is a void cell.
    //
    // With non-void terrain on top of the portal, just show our side and stay on the current
    // frame as far as FOV is concerned.
    if let Some(dest) = query::visible_portal(w, loc) {
        ret.insert(0, dest - offset);
    }

    Some(ret)
}

fn sight_fov_advance(w: &World,
                     offset: Point2D<i32>,
                     &(ref origin, ref prev_offset): &(Location, Point2D<i32>))
                     -> Option<(Location, Point2D<i32>)> {
    if query::terrain(w, *origin + *prev_offset).blocks_sight() {
        return None;
    }

    let loc = *origin + offset;

    let mut new_origin = *origin;

    if let Some(dest) = query::visible_portal(w, loc) {
        new_origin = dest - offset;
    }

    Some((new_origin, offset))
}

/// Return the field of view chart for drawing a screen.
pub fn screen_fov<F>(w: &World,
                  origin: Location,
                  is_valid_pos: F)
                  -> HashMap<Point2D<i32>, Vec<Location>>
    where F: Fn(Point2D<i32>) -> bool
{
    // TODO: HexFov won't type without type for is_wall_f?
    let fov = HexFov::new(vec![origin], move |offset, a| screen_fov_advance(w, &is_valid_pos, offset, a));
    HashMap::from_iter(fov)
}

/// Return the field of view chart for visible tiles.
pub fn sight_fov(w: &World, origin: Location, range: u32) -> HashSet<Location> {
    HashSet::from_iter(HexFov::new((origin, Point2D::new(0, 0)),
                                   |offset, a| if offset.hex_dist() as u32 > range {
                                       None
                                   } else {
                                       sight_fov_advance(w, offset, a)
                                   })
        // TODO: Get fake-isometric hack working in calx-grid.
        //.fake_isometric(move |offset, &(ref origin, _)| query::terrain(w, *origin + offset).form == terrain::Form::Wall)
        .map(|(offset, (ref origin, _))| *origin + offset))
}
