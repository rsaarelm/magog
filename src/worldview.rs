use time;
use calx;
use calx::{V2};
use calx::{Context, Rgb};
use calx::color::*;
use world::terrain;
use world::{TerrainType, Location, Chart};
use world::{FovStatus, Seen, Remembered};
use world::{Entity};
use viewutil::{SCREEN_W, SCREEN_H, chart_to_view, cells_on_screen};
use viewutil::{FLOOR_Z, BLOCK_Z};
use drawable::{Drawable};
use tilecache;
use tilecache::tile::*;

pub fn draw_world<C: Chart>(chart: &C, ctx: &mut calx::Context) {
    for &pt in cells_on_screen().iter() {
        let screen_pos = chart_to_view(pt) + V2(SCREEN_W / 2, SCREEN_H / 2);

        let loc = *chart + pt;
        let cell_drawable = CellDrawable::new(loc, Some(Seen));
        cell_drawable.draw(ctx, screen_pos);
    }
}

/// Drawable representation of a single map location.
pub struct CellDrawable {
    loc: Location,
    fov: Option<FovStatus>,
}

impl Drawable for CellDrawable {
    fn draw(&self, ctx: &mut Context, offset: V2<int>) {
        match self.fov {
            Some(_) => {
                self.draw_cell(ctx, offset)
            }
            None => {
                let (front_of_wall, is_door) = classify(self);
                if front_of_wall && !is_door {
                    self.draw_tile(ctx, CUBE, offset, BLOCK_Z, &BLACK);
                } else if !front_of_wall {
                    self.draw_tile(ctx, BLOCK_DARK, offset, BLOCK_Z, &BLACK);
                }
            }
        }

        fn classify(c: &CellDrawable) -> (bool, bool) {
            let mut front_of_wall = false;
            let mut is_door = false;
            let nw = c.loc + V2(-1, 0);
            let ne = c.loc + V2(0, -1);

            for &p in vec![nw, ne].iter() {
                let t = p.terrain();
                if t.is_wall() {
                    front_of_wall = true;
                    if t.is_walkable() { is_door = true; }
                }
            }
            (front_of_wall, is_door)
        }
    }
}

impl CellDrawable {
    pub fn new(loc: Location, fov: Option<FovStatus>) -> CellDrawable {
        CellDrawable {
            loc: loc,
            fov: fov,
        }
    }

    fn draw_tile(&self, ctx: &mut Context, idx: uint, offset: V2<int>, z: f32, color: &Rgb) {
        let color = match self.fov {
            Some(Remembered) => Rgb::new(0x22u8, 0x22u8, 0x11u8),
            _ => *color,
        };
        ctx.draw_image(offset, z, tilecache::get(idx), &color);
    }

    fn draw_cell(&self, ctx: &mut Context, offset: V2<int>) {
        self.draw_terrain(ctx, offset);

        if self.fov == Some(Seen) {
            // TODO
            /*
            for mob in self.loc.entities().iter() {
                self.draw_mob(ctx, offset, mob);
            }
            */
        }
    }

    fn draw_terrain(&self, ctx: &mut Context, offset: V2<int>) {
        let k = Kernel::new(|loc| loc.terrain(), self.loc);
        match k.center {
            terrain::Void => {
                self.draw_tile(ctx, BLANK_FLOOR, offset, FLOOR_Z, &BLACK);
            },
            terrain::Water => {
                self.draw_tile(ctx, WATER, offset, FLOOR_Z, &ROYALBLUE);
            },
            terrain::Shallows => {
                self.draw_tile(ctx, SHALLOWS, offset, FLOOR_Z, &CORNFLOWERBLUE);
            },
            terrain::Magma => {
                self.draw_tile(ctx, MAGMA, offset, FLOOR_Z, &DARKRED);
            },
            terrain::Tree => {
                // A two-toner, with floor, using two z-layers
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, TREE_TRUNK, offset, BLOCK_Z, &SADDLEBROWN);
                self.draw_tile(ctx, TREE_FOLIAGE, offset, BLOCK_Z, &GREEN);
            },
            terrain::Floor => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
            },
            terrain::Chasm => {
                self.draw_tile(ctx, CHASM, offset, FLOOR_Z, &DARKSLATEGRAY);
            },
            terrain::Grass => {
                self.draw_tile(ctx, GRASS, offset, FLOOR_Z, &DARKGREEN);
            },
            terrain::Downstairs => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, DOWNSTAIRS, offset, BLOCK_Z, &SLATEGRAY);
            },
            terrain::Portal => {
                let glow = (127.0 *(1.0 + (time::precise_time_s()).sin())) as u8;
                let portal_col = Rgb::new(glow, glow, 255);
                self.draw_tile(ctx, PORTAL, offset, BLOCK_Z, &portal_col);
            },
            terrain::Rock => {
                blockform(self, ctx, &k, offset, BLOCK, &DARKGOLDENROD);
            }
            terrain::Wall => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                wallform(self, ctx, &k, offset, WALL, &LIGHTSLATEGRAY, true);
            },
            terrain::RockWall => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                wallform(self, ctx, &k, offset, ROCKWALL, &LIGHTSLATEGRAY, true);
            },
            terrain::Fence => {
                // The floor type beneath the fence tile is visible, make it grass
                // if there's grass behind the fence. Otherwise make it regular
                // floor.
                if k.n == terrain::Grass || k.ne == terrain::Grass || k.nw == terrain::Grass {
                    self.draw_tile(ctx, GRASS, offset, FLOOR_Z, &DARKGREEN);
                } else {
                    self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                }
                wallform(self, ctx, &k, offset, FENCE, &DARKGOLDENROD, false);
            },
            terrain::Bars => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                wallform(self, ctx, &k, offset, BARS, &GAINSBORO, false);
            },
            terrain::Stalagmite => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, STALAGMITE, offset, BLOCK_Z, &DARKGOLDENROD);
            },
            terrain::Window => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                wallform(self, ctx, &k, offset, WINDOW, &LIGHTSLATEGRAY, false);
            },
            terrain::Door => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                wallform(self, ctx, &k, offset, DOOR, &LIGHTSLATEGRAY, true);
                wallform(self, ctx, &k, offset, DOOR + 4, &SADDLEBROWN, false);
            },
            terrain::OpenDoor => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                wallform(self, ctx, &k, offset, DOOR, &LIGHTSLATEGRAY, true);
            },
            terrain::Table => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, TABLE, offset, BLOCK_Z, &DARKGOLDENROD);
            },
            terrain::Fountain => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, FOUNTAIN, offset, BLOCK_Z, &GAINSBORO);
            },
            terrain::Altar => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, ALTAR, offset, BLOCK_Z, &GAINSBORO);
            },
            terrain::Barrel => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, BARREL, offset, BLOCK_Z, &DARKGOLDENROD);
            },
            terrain::Grave => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, GRAVE, offset, BLOCK_Z, &SLATEGRAY);
            },
            terrain::Stone => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, STONE, offset, BLOCK_Z, &SLATEGRAY);
            },
            terrain::Menhir => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, MENHIR, offset, BLOCK_Z, &SLATEGRAY);
            },
            terrain::DeadTree => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, TREE_TRUNK, offset, BLOCK_Z, &SADDLEBROWN);
            },
            terrain::TallGrass => {
                self.draw_tile(ctx, TALLGRASS, offset, BLOCK_Z, &GOLD);
            },
        }

        fn blockform(c: &CellDrawable, ctx: &mut Context, k: &Kernel<TerrainType>, offset: V2<int>, idx: uint, color: &Rgb) {
            c.draw_tile(ctx, idx, offset, BLOCK_Z, color);
            // Back lines for blocks with open floor behind them.
            if !k.nw.is_wall() {
                c.draw_tile(ctx, BLOCK_NW, offset, BLOCK_Z, color);
            }
            if !k.n.is_wall() {
                c.draw_tile(ctx, BLOCK_N, offset, BLOCK_Z, color);
            }
            if !k.ne.is_wall() {
                c.draw_tile(ctx, BLOCK_NE, offset, BLOCK_Z, color);
            }
        }

        fn wallform(c: &CellDrawable, ctx: &mut Context, k: &Kernel<TerrainType>, offset: V2<int>, idx: uint, color: &Rgb, opaque: bool) {
            let (left_wall, right_wall, block) = wall_flags_lrb(k);
            if block {
                if opaque {
                    c.draw_tile(ctx, CUBE, offset, BLOCK_Z, color);
                } else {
                    c.draw_tile(ctx, idx + 2, offset, BLOCK_Z, color);
                    return;
                }
            }
            if left_wall && right_wall {
                c.draw_tile(ctx, idx + 2, offset, BLOCK_Z, color);
            } else if left_wall {
                c.draw_tile(ctx, idx, offset, BLOCK_Z, color);
            } else if right_wall {
                c.draw_tile(ctx, idx + 1, offset, BLOCK_Z, color);
            } else if !block || !k.s.is_wall() {
                // NB: This branch has some actual local kernel logic not
                // handled by wall_flags_lrb.
                c.draw_tile(ctx, idx + 3, offset, BLOCK_Z, color);
            }
        }

        // Return code:
        // (there is a wall piece to the left front of the tile,
        //  there is a wall piece to the right front of the tile,
        //  there is a solid block in the tile)
        fn wall_flags_lrb(k: &Kernel<TerrainType>) -> (bool, bool, bool) {
            if k.nw.is_wall() && k.n.is_wall() && k.ne.is_wall() {
                // If there is open space to east or west, even if this block
                // has adjacent walls to the southeast or the southwest, those
                // will be using thin wall sprites, so this block needs to have
                // the corresponding wall bit to make the wall line not have
                // gaps.
                (!k.w.is_wall() || !k.sw.is_wall(), !k.e.is_wall() || !k.se.is_wall(), true)
            } else {
                (k.nw.is_wall(), k.ne.is_wall(), false)
            }
        }
    }

    fn draw_mob(&self, ctx: &mut Context, offset: V2<int>, mob: &Entity) {
        println!("TODO draw_mob");
    /*
        let body_pos =
            if is_bobbing(mob) {
                offset + *(timing::cycle_anim(
                        0.3f64,
                        [V2(0, 0), V2(0, -1)]))
            } else { offset };

        let (icon, color) = visual(mob.mob_type());
        match mob.mob_type() {
            mobs::Serpent => {
                // Body
                self.draw_tile(ctx, 94, body_pos, BLOCK_Z, &color);
                // Ground mound
                self.draw_tile(ctx, 95, offset, BLOCK_Z, &color);
            }
            _ => {
                self.draw_tile(ctx, icon, body_pos, BLOCK_Z, &color);
            }
        }

        fn visual(t: MobType) -> (uint, Rgb) {
            let kind = mobs::MOB_KINDS[t as uint];
            (kind.sprite, kind.color)
        }

        fn is_bobbing(mob: &Entity) -> bool {
            if mob.mob_type() == mobs::Player { return false; }
            if !mob.is_active() { return false; }
            true
        }
    */
    }
}

/// 3x3 grid of terrain cells. Use this as the input for terrain tile
/// computation, which will need to consider the immediate vicinity of cells.
struct Kernel<C> {
    n: C,
    ne: C,
    e: C,
    nw: C,
    center: C,
    se: C,
    w: C,
    sw: C,
    s: C,
}

impl<C: Clone> Kernel<C> {
    pub fn new(get: |Location| -> C, loc: Location) -> Kernel<C> {
        Kernel {
            n: get(loc + V2(-1, -1)),
            ne: get(loc + V2(0, -1)),
            e: get(loc + V2(1, -1)),
            nw: get(loc + V2(-1, 0)),
            center: get(loc),
            se: get(loc + V2(1, 0)),
            w: get(loc + V2(-1, 1)),
            sw: get(loc + V2(0, 1)),
            s: get(loc + V2(1, 1)),
        }
    }
}
