//! Game world display utilities.

use crate::brush::Brush;
use crate::cache;
use crate::view::PhysicsVector;
use crate::Icon;
use calx::{Dir12, Dir6};
use euclid::vec3;
use std::sync::Arc;
use world::{terrain, Location, Query, Terrain, TerrainQuery, World};

/// Surface angle for a visible sprite, used for dynamic lighting.
///
/// ```notrust
///
///      # north #
///     n         n
///    w           e
///
///  # x_         _y #
///   s  -x_   _y-  s
///    w    -*-    e
///       y-   -x
///      # south #
/// ```
#[derive(Copy, Eq, PartialEq, Clone, Debug)]
pub enum Angle {
    Up,
    North,
    XWallBack,
    Northeast,
    East,
    Southeast,
    YWall,
    South,
    XWall,
    Southwest,
    West,
    Northwest,
    YWallBack,
}

impl Angle {
    fn degree(self) -> Option<f32> {
        use self::Angle::*;
        Some(match self {
            North => 0.0,
            XWallBack => 30.0,
            Northeast => 60.0,
            East => 90.0,
            Southeast => 120.0,
            YWall => 150.0,
            South => 180.0,
            XWall => 210.0,
            Southwest => 240.0,
            West => 270.0,
            Northwest => 300.0,
            YWallBack => 330.0,
            _ => {
                return None;
            }
        })
    }

    /// Return the wall normal in physics space.
    ///
    /// Physics space x is right on screen, y is up on screen and z is towards viewer.
    pub fn normal(self) -> PhysicsVector {
        if let Some(deg) = self.degree() {
            let rad = deg.to_radians();
            vec3(rad.sin(), rad.cos(), 0.0)
        } else {
            debug_assert_eq!(self, Angle::Up);
            vec3(0.0, 0.0, 1.0)
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
/// Draw layer for visual map elements.
pub enum Layer {
    /// Floor sprites, below other map forms.
    Floor,
    /// Blood splatters etc. on floor.
    Decal,
    /// Large map objects, walls, most entities etc.
    Object,
    /// Transient effects shown on top of other map content.
    Effect,
}

/// Draw a blobform tile.
///
/// Set `is_solid` to true if the blob is the dark background part that fills the visible volume of
/// the blob but doesn't have visible walls.
#[allow(clippy::cognitive_complexity)]
fn blobform<F>(kernel: &Kernel, brush: &Arc<Brush>, is_solid: bool, draw: &mut F)
where
    F: FnMut(Layer, Angle, &Arc<Brush>, usize),
{
    use self::Angle::*;
    // This part gets a little tricky. Basic idea is that there's an inner pointy-top
    // hex core and the blob hull will snap to that instead of the outer flat-top hex
    // edge if neither adjacent face to the outer hex vertex is connected to another
    // blob.
    //
    // Based on how the sprites split up, the processing is done in four vertical
    // segments.

    // Shape the blob based on neighbors that are also blob forms.
    let neighbors = kernel.blob_neighbors();

    // Do we snap to the outer vertices?
    let ne_vertex = !neighbors[0] || !neighbors[1];
    let e_vertex = !neighbors[1] || !neighbors[2];
    let se_vertex = !neighbors[2] || !neighbors[3];
    let sw_vertex = !neighbors[3] || !neighbors[4];
    let w_vertex = !neighbors[4] || !neighbors[5];
    let nw_vertex = !neighbors[5] || !neighbors[0];

    // Show exposed faces if neighbors are not blob and not wall.
    let faces = if is_solid {
        [true, true, true, true, true, true]
    } else {
        kernel.blob_faces()
    };

    // Segment 2, middle left
    {
        if faces[0] {
            if nw_vertex && ne_vertex {
                draw(Layer::Object, North, brush, 7);
            } else if nw_vertex {
                draw(Layer::Object, XWallBack, brush, 15);
            } else {
                draw(Layer::Object, YWallBack, brush, 11);
            }
        }
        if faces[3] {
            if sw_vertex && se_vertex {
                draw(Layer::Object, South, brush, 19);
            } else if sw_vertex {
                draw(Layer::Object, YWall, brush, 23);
            } else {
                draw(Layer::Object, XWall, brush, 27);
            }
        }
    }

    // Segment 3, middle right
    {
        if faces[0] {
            if ne_vertex && nw_vertex {
                draw(Layer::Object, North, brush, 8);
            } else if ne_vertex {
                draw(Layer::Object, YWallBack, brush, 12);
            } else {
                draw(Layer::Object, XWallBack, brush, 16);
            }
        }
        if faces[3] {
            if se_vertex && sw_vertex {
                draw(Layer::Object, South, brush, 20);
            } else if se_vertex {
                draw(Layer::Object, XWall, brush, 28);
            } else {
                draw(Layer::Object, YWall, brush, 24);
            }
        }
    }

    // The side segments need to come after the middle
    // segments so that the vertical edges can overwrite the
    // middle segment pixels.

    // Segment 1, left edge
    {
        if w_vertex {
            if faces[5] {
                if nw_vertex {
                    draw(Layer::Object, Northwest, brush, 6);
                } else {
                    draw(Layer::Object, YWallBack, brush, 10);
                }
            }

            if faces[4] {
                if sw_vertex {
                    draw(Layer::Object, Southwest, brush, 18);
                } else {
                    draw(Layer::Object, XWall, brush, 26);
                }
            }
        } else if !is_solid && (faces[4] || faces[5]) {
            // Draw the left vertical line.
            draw(Layer::Object, West, brush, 2);
            if !faces[0] && nw_vertex {
                draw(Layer::Object, West, brush, 0);
            }
            if !faces[3] && sw_vertex {
                draw(Layer::Object, West, brush, 4);
            }
        }
    }

    // Segment 4, right edge
    {
        if e_vertex {
            if faces[1] {
                if ne_vertex {
                    draw(Layer::Object, Northeast, brush, 9);
                } else {
                    draw(Layer::Object, XWallBack, brush, 17);
                }
            }

            if faces[2] {
                if se_vertex {
                    draw(Layer::Object, Southeast, brush, 21);
                } else {
                    draw(Layer::Object, YWall, brush, 25);
                }
            }
        } else if !is_solid && (faces[1] || faces[2]) {
            // Draw the right vertical line.
            draw(Layer::Object, East, brush, 3);
            if !faces[0] && ne_vertex {
                draw(Layer::Object, East, brush, 1);
            }
            if !faces[3] && se_vertex {
                draw(Layer::Object, East, brush, 5);
            }
        }
    }
}

pub fn draw_terrain_sprites<F>(w: &World, loc: Location, mut draw: F)
where
    F: FnMut(Layer, Angle, &Arc<Brush>, usize),
{
    use self::Angle::*;

    let terrain = w.visual_terrain(loc);
    let kernel = Kernel::new(w, loc);
    let brush = cache::terrain(terrain);

    match terrain.form() {
        terrain::Form::Void | terrain::Form::Floor => {
            draw(Layer::Floor, Up, &brush, 0);
        }
        terrain::Form::Gate => {
            if let Some(d12) = Dir12::away_from(kernel.walk_mask()) {
                draw(Layer::Floor, Up, &brush, d12 as usize + 1);
            } else {
                draw(Layer::Floor, Up, &brush, 0);
            }
        }
        terrain::Form::Prop => {
            draw(Layer::Floor, Up, &cache::terrain(Terrain::Empty), 0);

            draw(Layer::Object, South, &brush, 0);
        }
        terrain::Form::Blob => {
            draw(Layer::Floor, Up, &cache::terrain(Terrain::Empty), 0);

            // Draw the solid blob first to block out other stuff.
            let solid = cache::misc(Icon::SolidBlob);
            blobform(&kernel, &solid, true, &mut draw);
            // Then draw the decoration with the actual brush.
            blobform(&kernel, &brush, false, &mut draw);
        }
        terrain::Form::Wall => {
            draw(Layer::Floor, Up, &cache::terrain(Terrain::Empty), 0);

            let extends = kernel.wall_extends();
            if extends[0] {
                draw(Layer::Object, XWall, &brush, 2);
            } else {
                draw(Layer::Object, XWall, &brush, 0);
            }
            if extends[1] {
                draw(Layer::Object, YWall, &brush, 3);
            } else {
                draw(Layer::Object, YWall, &brush, 1);
            }
        }
    }
    // TODO: Generate sprites for entities, with tweening state projection.
    // TODO: Generate special effect sprites grounded on this location.
}

#[derive(Clone)]
pub struct Kernel {
    pub n: Terrain,
    pub ne: Terrain,
    pub nw: Terrain,
    pub center: Terrain,
    pub se: Terrain,
    pub sw: Terrain,
    pub s: Terrain,
}

fn neighbor(w: &World, loc: Location, dir: Dir6) -> Terrain {
    let loc = w
        .visible_portal(loc + dir.to_v2())
        .unwrap_or(loc + dir.to_v2());
    w.visual_terrain(loc)
}

impl Kernel {
    pub fn new(w: &World, loc: Location) -> Kernel {
        Kernel {
            n: neighbor(w, loc, Dir6::North),
            ne: neighbor(w, loc, Dir6::Northeast),
            nw: neighbor(w, loc, Dir6::Northwest),
            center: w.visual_terrain(loc),
            se: neighbor(w, loc, Dir6::Southeast),
            sw: neighbor(w, loc, Dir6::Southwest),
            s: neighbor(w, loc, Dir6::South),
        }
    }

    /// Bool is true if left/right half of wall should be extended.
    pub fn wall_extends(&self) -> [bool; 2] { [self.nw.is_wall(), self.ne.is_wall()] }

    /// Bool is true if n/ne/se/s/sw/nw face of block is facing open air.
    pub fn blob_faces(&self) -> [bool; 6] {
        [
            !self.n.is_hull(),
            !self.ne.is_hull(),
            !self.se.is_hull(),
            !self.s.is_hull(),
            !self.sw.is_hull(),
            !self.nw.is_hull(),
        ]
    }

    pub fn blob_neighbors(&self) -> [bool; 6] {
        [
            !self.n.is_blob(),
            !self.ne.is_blob(),
            !self.se.is_blob(),
            !self.s.is_blob(),
            !self.sw.is_blob(),
            !self.nw.is_blob(),
        ]
    }

    /// Mark neighbors that are walkable terrain as true.
    pub fn walk_mask(&self) -> [bool; 6] {
        [
            !self.n.blocks_walk(),
            !self.ne.blocks_walk(),
            !self.se.blocks_walk(),
            !self.s.blocks_walk(),
            !self.sw.blocks_walk(),
            !self.nw.blocks_walk(),
        ]
    }
}
