//! Game world display utilities.

use std::sync::Arc;
use calx_resource::Resource;
use calx_grid::{Dir12, Dir6};
use world::{Brush, Location, World};
use world::{Query, terrain};

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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
/// Draw layer for visual map elements.
pub enum Layer {
    /// Floor sprites, below other map forms.
    Floor,
    /// Blood splatters etc. on floor.
    Decal,
    /// Small items on floor,.
    Items,
    /// Large map objects, walls, most entities etc.
    Object,
    /// Transient effects shown on top of other map content.
    Effect,
    /// Text captions for map elements, on top of everything else.
    Text,
}

pub fn draw_terrain_sprites<F>(w: &World, loc: Location, mut draw: F)
    where F: FnMut(Layer, Angle, &Resource<Brush>, usize)
{
    use self::Angle::*;

    let terrain = w.terrain(loc);
    let kernel = Kernel::new(w, loc);

    match terrain.form {
        terrain::Form::Void | terrain::Form::Floor => {
            draw(Layer::Floor, Up, &terrain.brush, 0);
        }
        terrain::Form::Gate => {
            if let Some(d12) = Dir12::away_from(&kernel.void_mask()) {
                draw(Layer::Floor, Up, &terrain.brush, d12 as usize + 1);
            } else {
                draw(Layer::Floor, Up, &terrain.brush, 0);
            }
        }
        terrain::Form::Prop => {
            draw(Layer::Object, South, &terrain.brush, 0);
        }
        terrain::Form::Blob => {
            // This part gets a little tricky. Basic idea is that there's an inner pointy-top
            // hex core and the blob hull will snap to that instead of the outer flat-top hex
            // edge if neither adjacent face to the outer hex vertex is connected to another
            // blob.
            //
            // Based on how the sprites split up, the processing is done in four vertical
            // segments.

            let faces = kernel.blob_faces();

            // Do we snap to the outer vertices?
            let ne_vertex = !faces[0] || !faces[1];
            let e_vertex = !faces[1] || !faces[2];
            let se_vertex = !faces[2] || !faces[3];
            let sw_vertex = !faces[3] || !faces[4];
            let w_vertex = !faces[4] || !faces[5];
            let nw_vertex = !faces[5] || !faces[0];

            // Segment 2, middle left
            {
                if faces[0] {
                    if nw_vertex && ne_vertex {
                        draw(Layer::Object, North, &terrain.brush, 7);
                    } else if nw_vertex {
                        draw(Layer::Object, XWallBack, &terrain.brush, 15);
                    } else {
                        draw(Layer::Object, YWallBack, &terrain.brush, 11);
                    }
                }
                if faces[3] {
                    if sw_vertex && se_vertex {
                        draw(Layer::Object, South, &terrain.brush, 19);
                    } else if sw_vertex {
                        draw(Layer::Object, YWall, &terrain.brush, 23);
                    } else {
                        draw(Layer::Object, XWall, &terrain.brush, 27);
                    }
                }
            }

            // Segment 3, middle right
            {
                if faces[0] {
                    if ne_vertex && nw_vertex {
                        draw(Layer::Object, North, &terrain.brush, 8);
                    } else if ne_vertex {
                        draw(Layer::Object, YWallBack, &terrain.brush, 12);
                    } else {
                        draw(Layer::Object, XWallBack, &terrain.brush, 16);
                    }
                }
                if faces[3] {
                    if se_vertex && sw_vertex {
                        draw(Layer::Object, South, &terrain.brush, 20);
                    } else if se_vertex {
                        draw(Layer::Object, XWall, &terrain.brush, 28);
                    } else {
                        draw(Layer::Object, YWall, &terrain.brush, 24);
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
                            draw(Layer::Object, Northwest, &terrain.brush, 6);
                        } else {
                            draw(Layer::Object, YWallBack, &terrain.brush, 10);
                        }
                    }

                    if faces[4] {
                        if sw_vertex {
                            draw(Layer::Object, Southwest, &terrain.brush, 18);
                        } else {
                            draw(Layer::Object, XWall, &terrain.brush, 26);
                        }
                    }
                } else {
                    // Draw the left vertical line.
                    draw(Layer::Object, West, &terrain.brush, 2);
                    if !faces[0] {
                        draw(Layer::Object, West, &terrain.brush, 0);
                    }
                    if !faces[3] {
                        draw(Layer::Object, West, &terrain.brush, 4);
                    }
                }
            }

            // Segment 4, right edge
            {
                if e_vertex {
                    if faces[1] {
                        if ne_vertex {
                            draw(Layer::Object, Northeast, &terrain.brush, 9);
                        } else {
                            draw(Layer::Object, XWallBack, &terrain.brush, 17);
                        }
                    }

                    if faces[2] {
                        if se_vertex {
                            draw(Layer::Object, Southeast, &terrain.brush, 21);
                        } else {
                            draw(Layer::Object, YWall, &terrain.brush, 25);
                        }
                    }
                } else {
                    // Draw the right vertical line.
                    draw(Layer::Object, East, &terrain.brush, 3);
                    if !faces[0] {
                        draw(Layer::Object, East, &terrain.brush, 1);
                    }
                    if !faces[3] {
                        draw(Layer::Object, East, &terrain.brush, 5);
                    }
                }
            }
        }
        terrain::Form::Wall => {
            let extends = kernel.wall_extends();
            if extends[0] {
                draw(Layer::Object, XWall, &terrain.brush, 2);
            } else {
                draw(Layer::Object, XWall, &terrain.brush, 0);
            }
            if extends[1] {
                draw(Layer::Object, YWall, &terrain.brush, 3);
            } else {
                draw(Layer::Object, YWall, &terrain.brush, 1);
            }
        }
    }
    // TODO: Generate sprites for entities, with tweening state projection.
    // TODO: Generate special effect sprites grounded on this location.
}

#[derive(Clone)]
pub struct Kernel {
    pub n: Arc<terrain::Tile>,
    pub ne: Arc<terrain::Tile>,
    pub nw: Arc<terrain::Tile>,
    pub center: Arc<terrain::Tile>,
    pub se: Arc<terrain::Tile>,
    pub sw: Arc<terrain::Tile>,
    pub s: Arc<terrain::Tile>,
}

fn neighbor(w: &World, loc: Location, dir: Dir6) -> Arc<terrain::Tile> {
    let loc = w.visible_portal(loc + dir.to_v2()).unwrap_or(loc + dir.to_v2());
    w.terrain(loc)
}

impl Kernel {
    pub fn new(w: &World, loc: Location) -> Kernel {
        Kernel {
            n: neighbor(w, loc, Dir6::North),
            ne: neighbor(w, loc, Dir6::Northeast),
            nw: neighbor(w, loc, Dir6::Northwest),
            center: w.terrain(loc),
            se: neighbor(w, loc, Dir6::Southeast),
            sw: neighbor(w, loc, Dir6::Southwest),
            s: neighbor(w, loc, Dir6::South),
        }
    }

    /// Bool is true if left/right half of wall should be extended.
    pub fn wall_extends(&self) -> [bool; 2] { [self.nw.is_wall(), self.ne.is_wall()] }

    /// Bool is true if n/ne/se/s/sw/nw face of block is facing open air.
    pub fn blob_faces(&self) -> [bool; 6] {
        // Because they work a bit differently visually, back-side faces
        // are not drawn if there is any hull touching, front is only
        // not drawn if there's another blob.
        [!self.n.is_hull(),
         !self.ne.is_hull(),
         !self.se.is_blob(),
         !self.s.is_blob(),
         !self.sw.is_blob(),
         !self.nw.is_hull()]
    }

    /// Mark neighbors that are not void terrain as true.
    pub fn void_mask(&self) -> [bool; 6] {
        [self.n.form != terrain::Form::Void,
         self.ne.form != terrain::Form::Void,
         self.se.form != terrain::Form::Void,
         self.s.form != terrain::Form::Void,
         self.sw.form != terrain::Form::Void,
         self.nw.form != terrain::Form::Void]
    }
}
