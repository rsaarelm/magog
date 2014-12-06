use std::num::{FloatMath};
use std::collections::HashMap;
use time;
use calx::{V2};
use calx::{Context, Rgb};
use calx::color::*;
use calx::timing;
use world::TerrainType;
use world::{Location, Chart};
use world::{FovStatus, EntityKind, MobType, MOB_SPECS};
use world::{Entity};
use viewutil::{chart_to_screen, cells_on_screen};
use viewutil::{FLOOR_Z, BLOCK_Z};
use drawable::{Drawable};
use tilecache;
use tilecache::tile::*;

pub fn draw_world<C: Chart>(chart: &C, ctx: &mut Context, damage_timers: &HashMap<Entity, uint>) {
    for pt in cells_on_screen() {
        let screen_pos = chart_to_screen(pt);
        let loc = *chart + pt;
        let cell_drawable = CellDrawable::new(loc, loc.fov_status(), damage_timers);
        cell_drawable.draw(ctx, screen_pos);
    }
}

/// Drawable representation of a single map location.
pub struct CellDrawable<'a> {
    loc: Location,
    fov: Option<FovStatus>,
    damage_timers: &'a HashMap<Entity, uint>
}

impl<'a> Drawable for CellDrawable<'a> {
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
                    if !t.blocks_walk() { is_door = true; }
                }
            }
            (front_of_wall, is_door)
        }
    }
}

impl<'a> CellDrawable<'a> {
    pub fn new(
        loc: Location,
        fov: Option<FovStatus>,
        damage_timers: &'a HashMap<Entity, uint>) -> CellDrawable<'a> {
        CellDrawable {
            loc: loc,
            fov: fov,
            damage_timers: damage_timers,
        }
    }

    fn draw_tile(&'a self, ctx: &mut Context, idx: uint, offset: V2<int>, z: f32, color: &Rgb) {
        let color = match self.fov {
            Some(FovStatus::Remembered) => Rgb::new(0x22u8, 0x22u8, 0x11u8),
            _ => *color,
        };
        ctx.draw_image(offset, z, tilecache::get(idx), &color);
    }

    fn draw_cell(&'a self, ctx: &mut Context, offset: V2<int>) {
        self.draw_terrain(ctx, offset);

        if self.fov == Some(FovStatus::Seen) {
            for e in self.loc.entities().iter() {
                self.draw_entity(ctx, offset, e);
            }
        }
    }

    fn draw_terrain(&'a self, ctx: &mut Context, offset: V2<int>) {
        let k = Kernel::new(|loc| loc.terrain(), self.loc);
        match k.center {
            TerrainType::Void => {
                self.draw_tile(ctx, BLANK_FLOOR, offset, FLOOR_Z, &BLACK);
            },
            TerrainType::Water => {
                self.draw_tile(ctx, WATER, offset, FLOOR_Z, &ROYALBLUE);
            },
            TerrainType::Shallows => {
                self.draw_tile(ctx, SHALLOWS, offset, FLOOR_Z, &CORNFLOWERBLUE);
            },
            TerrainType::Magma => {
                self.draw_tile(ctx, MAGMA, offset, FLOOR_Z, &DARKRED);
            },
            TerrainType::Tree => {
                // A two-toner, with floor, using two z-layers
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, TREE_TRUNK, offset, BLOCK_Z, &SADDLEBROWN);
                self.draw_tile(ctx, TREE_FOLIAGE, offset, BLOCK_Z, &GREEN);
            },
            TerrainType::Floor => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
            },
            TerrainType::Chasm => {
                self.draw_tile(ctx, CHASM, offset, FLOOR_Z, &DARKSLATEGRAY);
            },
            TerrainType::Grass => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &DARKGREEN);
            },
            TerrainType::Grass2 => {
                self.draw_tile(ctx, GRASS, offset, FLOOR_Z, &DARKGREEN);
            },
            TerrainType::Downstairs => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, DOWNSTAIRS, offset, BLOCK_Z, &SLATEGRAY);
            },
            TerrainType::Portal => {
                let glow = (127.0 *(1.0 + (time::precise_time_s()).sin())) as u8;
                let portal_col = Rgb::new(glow, glow, 255);
                self.draw_tile(ctx, PORTAL, offset, BLOCK_Z, &portal_col);
            },
            TerrainType::Rock => {
                blockform(self, ctx, &k, offset, BLOCK, &DARKGOLDENROD);
            }
            TerrainType::Wall => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                wallform(self, ctx, &k, offset, WALL, &LIGHTSLATEGRAY, true);
            },
            TerrainType::RockWall => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                wallform(self, ctx, &k, offset, ROCKWALL, &LIGHTSLATEGRAY, true);
            },
            TerrainType::Fence => {
                // The floor type beneath the fence tile is visible, make it grass
                // if there's grass behind the fence. Otherwise make it regular
                // floor.
                if k.n == TerrainType::Grass || k.ne == TerrainType::Grass || k.nw == TerrainType::Grass {
                    self.draw_tile(ctx, GRASS, offset, FLOOR_Z, &DARKGREEN);
                } else {
                    self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                }
                wallform(self, ctx, &k, offset, FENCE, &DARKGOLDENROD, false);
            },
            TerrainType::Bars => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                wallform(self, ctx, &k, offset, BARS, &GAINSBORO, false);
            },
            TerrainType::Stalagmite => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, STALAGMITE, offset, BLOCK_Z, &DARKGOLDENROD);
            },
            TerrainType::Window => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                wallform(self, ctx, &k, offset, WINDOW, &LIGHTSLATEGRAY, false);
            },
            TerrainType::Door => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                wallform(self, ctx, &k, offset, DOOR, &LIGHTSLATEGRAY, true);
                wallform(self, ctx, &k, offset, DOOR + 4, &SADDLEBROWN, false);
            },
            TerrainType::OpenDoor => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                wallform(self, ctx, &k, offset, DOOR, &LIGHTSLATEGRAY, true);
            },
            TerrainType::Table => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, TABLE, offset, BLOCK_Z, &DARKGOLDENROD);
            },
            TerrainType::Fountain => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, FOUNTAIN, offset, BLOCK_Z, &GAINSBORO);
            },
            TerrainType::Altar => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, ALTAR, offset, BLOCK_Z, &GAINSBORO);
            },
            TerrainType::Barrel => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, BARREL, offset, BLOCK_Z, &DARKGOLDENROD);
            },
            TerrainType::Grave => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, GRAVE, offset, BLOCK_Z, &SLATEGRAY);
            },
            TerrainType::Stone => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, STONE, offset, BLOCK_Z, &SLATEGRAY);
            },
            TerrainType::Menhir => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, MENHIR, offset, BLOCK_Z, &SLATEGRAY);
            },
            TerrainType::DeadTree => {
                self.draw_tile(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, TREE_TRUNK, offset, BLOCK_Z, &SADDLEBROWN);
            },
            TerrainType::TallGrass => {
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
                let idx = if k.n.is_wall() && (!k.nw.is_wall() || !k.ne.is_wall()) {
                    // TODO: Walltile-specific XY-walls
                    XYWALL
                } else {
                    idx + 3
                };
                c.draw_tile(ctx, idx, offset, BLOCK_Z, color);
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

    fn draw_entity(&'a self, ctx: &mut Context, offset: V2<int>, entity: &Entity) {
        match entity.kind() {
            EntityKind::Mob(m) => {
                let body_pos =
                    if entity.is_bobbing() {
                        offset + *(timing::cycle_anim(
                                0.3f64,
                                &[V2(0, 0), V2(0, -1)]))
                    } else { offset };

                let (icon, mut color) = (
                    MOB_SPECS[m as uint].sprite,
                    MOB_SPECS[m as uint].color);

                if let Some(&t) = self.damage_timers.get(entity) {
                    color = if t % 2 == 0 { &WHITE } else { &BLACK };
                }

                if m == MobType::Serpent {
                    // Special case, Serpent sprite is made of two parts.
                    // Body
                    self.draw_tile(ctx, icon, body_pos, BLOCK_Z, color);
                    // Ground mound, doesn't bob.
                    self.draw_tile(ctx, icon + 1, offset, BLOCK_Z, color);
                } else {
                    self.draw_tile(ctx, icon, body_pos, BLOCK_Z, color);
                }
            }
            todo => { println!("TODO: Draw {} entity", todo) }
        }
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
