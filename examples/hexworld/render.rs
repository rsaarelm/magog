use calx::color::*;
use calx::{Rgba, Kernel, KernelTerrain};
use super::{Terrain};
use spr::{Spr};

pub trait RenderTerrain {
    fn render<F>(&self, draw: F)
        where F: FnMut(i8, Spr, Rgba, Rgba);
}

impl RenderTerrain for Kernel<Terrain> {
    fn render<F>(&self, mut draw: F)
        // Z-layer, sprite, fore-color, back-color
        where F: FnMut(i8, Spr, Rgba, Rgba)
    {
        static FLOOR: i8 = 1;
        static BLOCK: i8 = 0;
        use Terrain::*;

        fn wallform<C: KernelTerrain, F>(
            k: &Kernel<C>, draw: &mut F,
            short: Spr, short_color: Rgba, short_back: Rgba,
            long: Spr, long_color: Rgba, long_back: Rgba)
            where F: FnMut(i8, Spr, Rgba, Rgba)
        {
            let extends = k.wall_extends();
            if extends[0] {
                draw(BLOCK, long, long_color * 0.5, long_back);
            } else {
                draw(BLOCK, short, short_color * 0.5, short_back);
            }
            if extends[1] {
                draw(BLOCK, long + 1, long_color, long_back);
            } else {
                draw(BLOCK, short + 1, short_color, short_back);
            }
        }

        fn blockform<C: KernelTerrain, F>(
            k: &Kernel<C>, draw: &mut F,
            face: Spr, color: Rgba, back: Rgba)
            where F: FnMut(i8, Spr, Rgba, Rgba)
        {
            draw(0, Spr::FloorBlank, BLACK, BLACK);

            let faces = k.block_faces();

            if faces[5] { draw(BLOCK, Spr::BlockNW, color * 0.5, BLACK); }
            if faces[0] { draw(BLOCK, Spr::BlockN, color * 0.5, BLACK); }
            if faces[1] { draw(BLOCK, Spr::BlockNE, color * 0.5, BLACK); }
            if faces[4] { draw(BLOCK, face, color * 0.25, back * 0.25); }
            if faces[3] { draw(BLOCK, face + 1, color, back); }
            if faces[2] { draw(BLOCK, face + 2, color * 0.5, back * 0.5); }
        }

        match self.center {
            Void => draw(FLOOR, Spr::FloorBlank, MAGENTA, BLACK),
            Floor => draw(FLOOR, Spr::Floor, SLATEGRAY, BLACK),
            Grass => draw(FLOOR, Spr::GrassFloor, DARKGREEN, BLACK),
            Water => draw(FLOOR, Spr::WaterFloor, CYAN, ROYALBLUE),
            Magma => draw(FLOOR, Spr::WaterFloor, YELLOW, DARKRED),
            Tree => {
                draw(FLOOR, Spr::Floor, SLATEGRAY, BLACK);
                draw(BLOCK, Spr::TreeTrunk, SADDLEBROWN, BLACK);
                draw(BLOCK, Spr::Foliage, GREEN, BLACK);
            }

            Wall => {
                draw(FLOOR, Spr::Floor, SLATEGRAY, BLACK);
                wallform(self, &mut draw,
                         Spr::BrickWallShort, LIGHTSLATEGRAY, BLACK,
                         Spr::BrickWall, LIGHTSLATEGRAY, BLACK);
            }

            Door => {
                draw(FLOOR, Spr::Floor, SLATEGRAY, BLACK);
                wallform(self, &mut draw,
                         Spr::BrickWallShort, LIGHTSLATEGRAY, BLACK,
                         Spr::BrickOpenWall, LIGHTSLATEGRAY, BLACK);
                wallform(self, &mut draw,
                         Spr::DoorWallShort, SADDLEBROWN, BLACK,
                         Spr::DoorWall, SADDLEBROWN, BLACK);
            }

            Window => {
                draw(FLOOR, Spr::Floor, SLATEGRAY, BLACK);
                wallform(self, &mut draw,
                         Spr::BrickWallShort, LIGHTSLATEGRAY, BLACK,
                         Spr::BrickWindowWall, LIGHTSLATEGRAY, BLACK);
            }

            Rock => {
                blockform(self, &mut draw, Spr::BlockRock, DARKGOLDENROD, BLACK);
            }
        }
    }
}
