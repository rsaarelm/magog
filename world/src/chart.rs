use std::collections::HashMap;
use euclid::Point2D;
use std::iter::FromIterator;
use calx_grid::{HexFov, HexGeom};
use world::World;
use query;
use terrain::Form;
use location::Location;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
enum AccumVisibleState {
    Visible,
    VisibleEdge,
    Unseen,
}

/// Data store for a single cell in a chart.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Cell {
    visible: AccumVisibleState,
    /// Stack of origins, the first one is the active one.
    ///
    /// Origin locations after the first one are for the earlier portal regions that were travelled
    /// to get to the one for the current cell.
    pub origins: Vec<Location>,
}

impl Cell {
    fn new(origin: Location) -> Cell {
        Cell {
            visible: AccumVisibleState::Visible,
            origins: vec![origin],
        }
    }

    #[inline]
    pub fn is_visible(&self) -> bool {
        self.visible != AccumVisibleState::Unseen
    }

    fn advance(&self, w: &World, offset: Point2D<i32>) -> Cell {
        let loc = self.origins[0] + offset;
        let mut terrain = query::terrain(w, loc);

        let mut origins = self.origins.clone();
        // Go through a portal if terrain on our side of the portal is a void cell.
        //
        // With non-void terrain on top of the portal, just show our side and stay on the current
        // frame as far as FOV is concerned.
        if let Some(dest) = query::portal(w, loc) {
            if terrain.form == Form::Void {
                origins.insert(0, dest - offset);
                terrain = query::terrain(w, dest);
            }
        }

        let visible = match self.visible {
            AccumVisibleState::Visible if terrain.blocks_sight() => AccumVisibleState::VisibleEdge,
            AccumVisibleState::Visible => AccumVisibleState::Visible,
            _ => AccumVisibleState::Unseen,
        };


        Cell {
            visible: visible,
            origins: origins,
        }
    }
}

pub fn build(w: &World, origin: Location, range: u32) -> HashMap<Point2D<i32>, Cell> {
    HashMap::from_iter(HexFov::new(Cell::new(origin),
                                   |offset, a| if offset.hex_dist() as u32 > range {
                                       None
                                   } else {
                                       Some(a.advance(w, offset))
                                   })
                           .fake_isometric(|pos, a| {
                               query::terrain(w, a.origins[0] + pos).form == Form::Wall
                           }))
}
