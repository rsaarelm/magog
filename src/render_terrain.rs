use calx::color::*;
use calx::{Rgba, Kernel, KernelTerrain};
use calx::backend::{Image};
use content::{TerrainType, Brush};

pub trait RenderTerrain {
    /// Generate draw instructions for a terrain cell.
    ///
    /// Params to the draw function: Draw layer, brush, brush frame, main
    /// color, border color.
    fn render<F>(&self, draw: F)
        where F: FnMut(Image, Angle, Rgba, Rgba);
}

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
pub enum Angle {
    Up,
    North,
    Northeast,
    Southeast,
    YWall,
    South,
    XWall,
    Southwest,
    Northwest,
}

impl Angle {
    /// Return the angle of the vertical surface, if any, in degrees.
    pub fn degree(&self) -> Option<f32> {
        match *self {
            Angle::Up => None,
            Angle::North => Some(0.0),
            Angle::Northeast => Some(60.0),
            Angle::Southeast => Some(120.0),
            Angle::YWall => Some(150.0),
            Angle::South => Some(180.0),
            Angle::XWall => Some(210.0),
            Angle::Southwest => Some(240.0),
            Angle::Northwest => Some(300.0),
        }
    }
}

impl RenderTerrain for Kernel<TerrainType> {
    fn render<F>(&self, mut draw: F)
        where F: FnMut(Image, Angle, Rgba, Rgba)
    {
        use content::Brush::*;
        use self::Angle::*;

        enum T {
            Floor(Brush, Rgba),
            Floor2(Brush, Rgba, Rgba),
            Prop(Brush, Rgba),
            Prop2(Brush, Rgba, Rgba),
            Wall(Brush, Rgba),
            Wall2(Brush, Rgba, Rgba),
            Block(Brush, Rgba),
            Block2(Brush, Rgba, Rgba),
        }

        fn process<C: KernelTerrain, F>(
            k: &Kernel<C>, draw: &mut F, kind: T)
            where F: FnMut(Image, Angle, Rgba, Rgba)
        {
            match kind {
                T::Floor(brush, color) => process(k, draw, T::Floor2(brush, color, BLACK)),
                T::Prop(brush, color) => process(k, draw, T::Prop2(brush, color, BLACK)),
                T::Wall(brush, color) => process(k, draw, T::Wall2(brush, color, BLACK)),
                T::Block(brush, color) => process(k, draw, T::Block2(brush, color, BLACK)),

                T::Floor2(brush, color, back) => {
                    draw(brush.get(0), Up, color, back);
                }
                T::Prop2(brush, color, back) => {
                    draw(brush.get(0), South, color, back);
                }
                T::Wall2(brush, color, back) => {
                    let extends = k.wall_extends();
                    if extends[0] {
                        draw(brush.get(2), XWall, color, back);
                    } else {
                        draw(brush.get(0), XWall, color, back);
                    }
                    if extends[1] {
                        draw(brush.get(3), YWall, color, back);
                    } else {
                        draw(brush.get(1), YWall, color, back);
                    }
                }
                T::Block2(brush, color, back) => {
                    let faces = k.block_faces();

                    if faces[5] { draw(BlockRear.get(0), Northwest, color, BLACK); }
                    if faces[0] { draw(BlockRear.get(1), North, color, BLACK); }
                    if faces[1] { draw(BlockRear.get(2), Northeast, color, BLACK); }
                    if faces[4] { draw(brush.get(0), Southwest, color, back); }
                    if faces[3] { draw(brush.get(1), South, color, back); }
                    if faces[2] { draw(brush.get(2), Southeast, color, back); }
                }
            }
        }

        for i in match self.center {
            TerrainType::Void => vec![T::Floor(BlankFloor, MAGENTA)],
            TerrainType::Floor => vec![T::Floor(Floor, SLATEGRAY)],
            TerrainType::Water => vec![T::Floor(Water, ROYALBLUE)],
            TerrainType::Shallows => vec![T::Floor(Shallows, CORNFLOWERBLUE)],
            TerrainType::Magma => vec![T::Floor2(Water, YELLOW, DARKRED)],
            TerrainType::Downstairs => vec![T::Floor(StairsDown, SLATEGRAY)],
            TerrainType::Wall => vec![
                T::Floor(BlankFloor, SLATEGRAY),
                T::Wall(BrickWall, LIGHTSLATEGRAY),
            ],
            TerrainType::RockWall => vec![
                T::Floor(BlankFloor, SLATEGRAY),
                T::Wall(RockWall, LIGHTSLATEGRAY),
            ],
            TerrainType::Rock => vec![
                T::Floor(BlankFloor, SLATEGRAY),
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
                if self.n == TerrainType::Grass || self.ne == TerrainType::Grass || self.nw == TerrainType::Grass {
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
            process(self, &mut draw, i);
        }
    }
}
