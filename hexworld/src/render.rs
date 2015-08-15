use calx::color::*;
use calx::{Rgba, Kernel, KernelTerrain};
use super::{Terrain};
use brush::{Brush};

pub trait RenderTerrain {
    fn render<F>(&self, draw: F)
        where F: FnMut(i8, Brush, usize, Rgba, Rgba);
}

impl RenderTerrain for Kernel<Terrain> {
    fn render<F>(&self, mut draw: F)
        // Z-layer, sprite, fore-color, back-color
        where F: FnMut(i8, Brush, usize, Rgba, Rgba)
    {
        static FLOOR: i8 = 8;
        static BLOCK: i8 = 0;
        use Terrain::*;

        fn wallform<C: KernelTerrain, F>(
            k: &Kernel<C>, draw: &mut F,
            brush: Brush, short_color: Rgba, short_back: Rgba,
            long_color: Rgba, long_back: Rgba)
            where F: FnMut(i8, Brush, usize, Rgba, Rgba)
        {
            let extends = k.wall_extends();
            if extends[0] {
                draw(BLOCK, brush, 2, long_color * 0.5, long_back);
            } else {
                draw(BLOCK, brush, 0, short_color * 0.5, short_back);
            }
            if extends[1] {
                draw(BLOCK, brush, 3, long_color, long_back);
            } else {
                draw(BLOCK, brush, 1, short_color, short_back);
            }
        }

        fn blockform<C: KernelTerrain, F>(
            k: &Kernel<C>, draw: &mut F,
            face: Brush, color: Rgba, back: Rgba)
            where F: FnMut(i8, Brush, usize, Rgba, Rgba)
        {
            draw(0, Brush::FloorBlank, 0, BLACK, BLACK);

            let faces = k.block_faces();

            if faces[5] { draw(BLOCK, Brush::BlockRear, 0, color * 0.5, BLACK); }
            if faces[0] { draw(BLOCK, Brush::BlockRear, 1, color * 0.5, BLACK); }
            if faces[1] { draw(BLOCK, Brush::BlockRear, 2, color * 0.5, BLACK); }
            if faces[4] { draw(BLOCK, face, 0, color * 0.25, back * 0.25); }
            if faces[3] { draw(BLOCK, face, 1, color, back); }
            if faces[2] { draw(BLOCK, face, 2, color * 0.5, back * 0.5); }
        }

        match self.center {
            Void => draw(FLOOR, Brush::FloorBlank, 0, MAGENTA, BLACK),
            Floor => draw(FLOOR, Brush::Floor, 0, SLATEGRAY, BLACK),
            Grass => draw(FLOOR, Brush::GrassFloor, 0, DARKGREEN, BLACK),
            Water => draw(FLOOR, Brush::WaterFloor, 0, CYAN, ROYALBLUE),
            Magma => draw(FLOOR, Brush::WaterFloor, 0, YELLOW, DARKRED),
            Tree => {
                draw(FLOOR, Brush::Floor, 0, SLATEGRAY, BLACK);
                draw(BLOCK, Brush::TreeTrunk, 0, SADDLEBROWN, BLACK);
                draw(BLOCK, Brush::Foliage, 0, GREEN, BLACK);
            }

            Wall => {
                draw(FLOOR, Brush::Floor, 0, SLATEGRAY, BLACK);
                wallform(self, &mut draw,
                         Brush::BrickWall, LIGHTSLATEGRAY, BLACK,
                         LIGHTSLATEGRAY, BLACK);
            }

            Door => {
                draw(FLOOR, Brush::Floor, 0, SLATEGRAY, BLACK);
                wallform(self, &mut draw,
                         Brush::BrickOpenWall, LIGHTSLATEGRAY, BLACK,
                         LIGHTSLATEGRAY, BLACK);
                wallform(self, &mut draw,
                         Brush::DoorWall, SADDLEBROWN, BLACK,
                         SADDLEBROWN, BLACK);
            }

            Window => {
                draw(FLOOR, Brush::Floor, 0, SLATEGRAY, BLACK);
                wallform(self, &mut draw,
                         Brush::BrickWindowWall, LIGHTSLATEGRAY, BLACK,
                         LIGHTSLATEGRAY, BLACK);
            }

            Rock => {
                blockform(self, &mut draw, Brush::BlockRock, DARKGOLDENROD, BLACK);
            }
        }
    }
}
