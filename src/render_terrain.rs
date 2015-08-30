use calx::color::*;
use calx::{Rgba, Kernel, KernelTerrain};
use calx::backend::{Image};
use content::{TerrainType, Brush};

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

#[derive(Copy, Eq, PartialEq, Clone, Debug)]
pub enum Purpose {
    Element,
    Filler,
}

impl Angle {
    /// Return the angle of the vertical surface, if any, in degrees.
    pub fn degree(&self) -> Option<f32> {
        match *self {
            Angle::Up => None,
            Angle::North => Some(0.0),
            Angle::XWallBack => Some(30.0),
            Angle::Northeast => Some(60.0),
            Angle::East => Some(90.0),
            Angle::Southeast => Some(120.0),
            Angle::YWall => Some(150.0),
            Angle::South => Some(180.0),
            Angle::XWall => Some(210.0),
            Angle::Southwest => Some(240.0),
            Angle::West => Some(270.0),
            Angle::Northwest => Some(300.0),
            Angle::YWallBack => Some(330.0),
        }
    }
}

/// Generate draw instructions for a terrain cell.
///
/// Params to the draw function: Draw layer, brush, brush frame, main
/// color, border color.
pub fn render<F>(k: &Kernel<TerrainType>, mut draw: F)
    where F: FnMut(Image, Angle, Purpose, Rgba, Rgba)
{
    use content::Brush::*;
    use self::Angle::*;
    use self::Purpose::*;

    enum T {
        Floor(Brush, Rgba),
        Floor2(Brush, Rgba, Rgba),
        Prop(Brush, Rgba),
        Prop2(Brush, Rgba, Rgba),
        Wall(Brush, Rgba),
        Wall2(Brush, Rgba, Rgba),
        Block(Brush, Rgba),
        Block2(Brush, Rgba, Rgba),
        Filler(Brush),
    }

    fn process<C: KernelTerrain, F>(
        k: &Kernel<C>, draw: &mut F, kind: T)
        where F: FnMut(Image, Angle, Purpose, Rgba, Rgba)
        {
            match kind {
                T::Floor(brush, color) => process(k, draw, T::Floor2(brush, color, BLACK)),
                T::Prop(brush, color) => process(k, draw, T::Prop2(brush, color, BLACK)),
                T::Wall(brush, color) => process(k, draw, T::Wall2(brush, color, BLACK)),
                T::Block(brush, color) => process(k, draw, T::Block2(brush, color, BLACK)),

                T::Floor2(brush, color, back) => {
                    draw(brush.get(0), Up, Element, color, back);
                }
                T::Prop2(brush, color, back) => {
                    draw(brush.get(0), South, Element, color, back);
                }
                T::Wall2(brush, color, back) => {
                    let extends = k.wall_extends();
                    if extends[0] {
                        draw(brush.get(2), XWall, Element, color, back);
                    } else {
                        draw(brush.get(0), XWall, Element, color, back);
                    }
                    if extends[1] {
                        draw(brush.get(3), YWall, Element, color, back);
                    } else {
                        draw(brush.get(1), YWall, Element, color, back);
                    }
                }
                T::Block2(brush, color, back) => {

                    // This part gets a little tricky. Basic idea is that
                    // there's an inner pointy-top hex core and the block hull
                    // will snap to that instead of the outer flat-top hex
                    // edge if neither adjacent face to the outer hex vertex
                    // is connected to another block.
                    //
                    // Based on how the sprites split up, the processing is
                    // done in four vertical segments.

                    let faces = k.block_faces();

                    // Do we snap to the outer vertices?
                    let ne_vertex = !faces[0] || !faces[1];
                    let e_vertex  = !faces[1] || !faces[2];
                    let se_vertex = !faces[2] || !faces[3];
                    let sw_vertex = !faces[3] || !faces[4];
                    let w_vertex  = !faces[4] || !faces[5];
                    let nw_vertex = !faces[5] || !faces[0];

                    // Segment 2, middle left
                    {
                        if faces[0] {
                            if nw_vertex && ne_vertex {
                                draw(BlockRear.get(1), North, Element, color, BLACK);
                            } else if nw_vertex {
                                draw(BlockRear.get(9), XWallBack, Element, color, BLACK);
                            } else {
                                draw(BlockRear.get(5), YWallBack, Element, color, BLACK);
                            }
                        }
                        if faces[3] {
                            if sw_vertex && se_vertex {
                                draw(brush.get(1), South, Element, color, back);
                            } else if sw_vertex {
                                draw(brush.get(5), YWall, Element, color, back);
                            } else {
                                draw(brush.get(9), XWall, Element, color, back);
                            }
                        }
                    }

                    // Segment 3, middle right
                    {
                        if faces[0] {
                            if ne_vertex && nw_vertex {
                                draw(BlockRear.get(2), North, Element, color, BLACK);
                            } else if ne_vertex {
                                draw(BlockRear.get(6), YWallBack, Element, color, BLACK);
                            } else {
                                draw(BlockRear.get(10), XWallBack, Element, color, BLACK);
                            }
                        }
                        if faces[3] {
                            if se_vertex && sw_vertex {
                                draw(brush.get(2), South, Element, color, back);
                            } else if se_vertex {
                                draw(brush.get(10), XWall, Element, color, back);
                            } else {
                                draw(brush.get(6), YWall, Element, color, back);
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
                                    draw(BlockRear.get(0), Northwest, Element, color, BLACK);
                                } else {
                                    draw(BlockRear.get(4), YWallBack, Element, color, BLACK);
                                }
                            }

                            if faces[4] {
                                if sw_vertex {
                                    draw(brush.get(0), Southwest, Element, color, back);
                                } else {
                                    draw(brush.get(8), XWall, Element, color, back);
                                }
                            }
                        } else {
                            // Draw the left vertical line.
                            draw(BlockVertical.get(2), West, Element, color, BLACK);
                            if !faces[0] {
                                draw(BlockVertical.get(0), West, Element, color, BLACK);
                            }
                            if !faces[3] {
                                draw(BlockVertical.get(4), West, Element, color, BLACK);
                            }
                        }
                    }

                    // Segment 4, right edge
                    {
                        if e_vertex {
                            if faces[1] {
                                if ne_vertex {
                                    draw(BlockRear.get(3), Northeast, Element, color, BLACK);
                                } else {
                                    draw(BlockRear.get(11), XWallBack, Element, color, BLACK);
                                }
                            }

                            if faces[2] {
                                if se_vertex {
                                    draw(brush.get(3), Southeast, Element, color, back);
                                } else {
                                    draw(brush.get(7), YWall, Element, color, back);
                                }
                            }
                        } else {
                            // Draw the right vertical line.
                            draw(BlockVertical.get(3), East, Element, color, BLACK);
                            if !faces[0] {
                                draw(BlockVertical.get(1), East, Element, color, BLACK);
                            }
                            if !faces[3] {
                                draw(BlockVertical.get(5), East, Element, color, BLACK);
                            }
                        }
                    }
                }
                T::Filler(brush) => {
                    draw(brush.get(0), Up, Filler, BLACK, BLACK);
                }
            }
        }

    for i in match k.center {
        TerrainType::Void => vec![T::Floor(BlankFloor, MAGENTA)],
        TerrainType::Floor => vec![T::Floor(Floor, SLATEGRAY)],
        TerrainType::Water => vec![T::Floor(Water, ROYALBLUE)],
        TerrainType::Shallows => vec![T::Floor(Shallows, CORNFLOWERBLUE)],
        TerrainType::Magma => vec![T::Floor2(Water, YELLOW, DARKRED)],
        TerrainType::Downstairs => vec![T::Floor(StairsDown, SLATEGRAY)],
        TerrainType::Wall => vec![
            T::Filler(BlankFloor),
            T::Wall(BrickWall, LIGHTSLATEGRAY),
        ],
        TerrainType::RockWall => vec![
            T::Filler(BlankFloor),
            T::Wall(RockWall, LIGHTSLATEGRAY),
        ],
        TerrainType::Rock => vec![
            T::Filler(BlankFloor),
            T::Block(BlockRock, DARKGOLDENROD),
        ],
        TerrainType::Tree => vec![
            T::Floor(BlankFloor, SLATEGRAY),
            T::Prop(TreeTrunk, SADDLEBROWN),
            T::Prop(TreeFoliage, GREEN),
        ],
        TerrainType::Grass => vec![T::Floor(Floor, DARKGREEN)],
        TerrainType::Grass2 => vec![ T::Floor(Grass, DARKGREEN)],
        TerrainType::Stalagmite => vec![
            T::Floor(BlankFloor, SLATEGRAY),
            T::Prop(Stalagmite, DARKGOLDENROD),
        ],
        TerrainType::Door => vec![
            T::Floor(BlankFloor, SLATEGRAY),
            T::Wall(BrickOpenWall, LIGHTSLATEGRAY),
            T::Wall(DoorWall, SADDLEBROWN),
        ],
        TerrainType::OpenDoor => vec![
            T::Floor(BlankFloor, SLATEGRAY),
            T::Wall(BrickOpenWall, LIGHTSLATEGRAY),
        ],
        TerrainType::Window => vec![
            T::Floor(BlankFloor, SLATEGRAY),
            T::Wall(BrickWindowWall, LIGHTSLATEGRAY),
        ],
        TerrainType::Table => vec![
            T::Floor(BlankFloor, SLATEGRAY),
            T::Prop(Table, DARKGOLDENROD),
        ],
        TerrainType::Fence => vec![
            // The floor type beneath the fence tile is visible, make it grass
            // if there's grass behind the fence. Otherwise make it regular
            // floor.
            if k.n == TerrainType::Grass || k.ne == TerrainType::Grass || k.nw == TerrainType::Grass {
                T::Floor(Grass, GREEN)
            } else {
                T::Floor(Floor, SLATEGRAY)
            },
            T::Wall(FenceWall, DARKGOLDENROD),
        ],
        TerrainType::Bars => vec![
            T::Floor(BlankFloor, SLATEGRAY),
            T::Wall(BarsWall, GAINSBORO),
        ],
        TerrainType::Fountain => vec![
            T::Floor(BlankFloor, SLATEGRAY),
            T::Prop(Table, DARKGOLDENROD),
        ],
        TerrainType::Altar => vec![
            T::Floor(BlankFloor, SLATEGRAY),
            T::Prop(Fountain, GAINSBORO),
        ],
        TerrainType::Barrel => vec![
            T::Floor(BlankFloor, SLATEGRAY),
            T::Prop(Barrell, DARKGOLDENROD),
        ],
        TerrainType::Grave => vec![
            T::Floor(BlankFloor, SLATEGRAY),
            T::Prop(Grave, SLATEGRAY),
        ],
        TerrainType::Stone => vec![
            T::Floor(BlankFloor, SLATEGRAY),
            T::Prop(Stone, SLATEGRAY),
        ],
        TerrainType::Menhir => vec![
            T::Floor(BlankFloor, SLATEGRAY),
            T::Prop(Menhir, SLATEGRAY),
        ],
        TerrainType::DeadTree => vec![
            T::Floor(BlankFloor, SLATEGRAY),
            T::Prop(TreeTrunk, SADDLEBROWN),
        ],
    }.into_iter() {
        process(k, &mut draw, i);
    }
}
