use std::collections::HashMap;
use time;
use util::{V2, Rgb, timing};
use util::color::*;
use backend::{Canvas, CanvasUtil};
use world::TerrainType;
use world::{Location, Chart};
use world::{FovStatus};
use world::{Entity};
use world::{Light};
use viewutil::{chart_to_screen, cells_on_screen, level_z_to_view};
use viewutil::{FLOOR_Z, BLOCK_Z, DEPTH_Z_MODIFIER, PIXEL_UNIT};
use drawable::{Drawable};
use tilecache;
use tilecache::tile::*;

pub fn draw_world<C: Chart+Copy>(chart: &C, ctx: &mut Canvas, damage_timers: &HashMap<Entity, u32>) {
    for pt in cells_on_screen() {
        let screen_pos = chart_to_screen(pt);
        let loc = *chart + pt;
        let cell_drawable = CellDrawable::new(
            loc, 0, loc.fov_status(), loc.light(), damage_timers);
        cell_drawable.draw(ctx, screen_pos);
    }
}

/// Drawable representation of a single map location.
pub struct CellDrawable<'a> {
    pub loc: Location,
    pub depth: i32,
    pub fov: Option<FovStatus>,
    pub light: Light,
    damage_timers: &'a HashMap<Entity, u32>,
}

impl<'a> Drawable for CellDrawable<'a> {
    fn draw(&self, ctx: &mut Canvas, offset: V2<f32>) {
        match self.fov {
            Some(_) => {
                self.draw_cell(ctx, offset)
            }
            None => {
                let (front_of_wall, is_door) = classify(self);
                if front_of_wall && !is_door {
                    self.draw_tile(ctx, CUBE, offset, BLOCK_Z, &BLACK);
                } else if !front_of_wall {
                    self.draw_tile(ctx, BLANK_FLOOR, offset, FLOOR_Z, &BLACK);
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
        depth: i32,
        fov: Option<FovStatus>,
        light: Light,
        damage_timers: &'a HashMap<Entity, u32>) -> CellDrawable<'a> {
        CellDrawable {
            loc: loc,
            depth: depth,
            fov: fov,
            light: light,
            damage_timers: damage_timers,
        }
    }

    fn draw_tile(&'a self, ctx: &mut Canvas, idx: usize, offset: V2<f32>, z: f32, color: &Rgb) {
        self.draw_tile2(ctx, idx, offset, z, color, &BLACK);
    }

    /// Draw edge lines to floor tile if there are chasm tiles to the back.
    fn floor_edges(&'a self, ctx: &mut Canvas, offset: V2<f32>, color: &Rgb) {
        // Shift edge offset from block top level to floor level.
        let offset = offset + V2(0, PIXEL_UNIT / 2).map(|x| x as f32);

        if (self.loc + V2(-1, -1)).terrain().is_hole() {
            self.draw_tile(ctx, BLOCK_N, offset, FLOOR_Z, color);
        }
        if (self.loc + V2(-1, 0)).terrain().is_hole() {
            self.draw_tile(ctx, BLOCK_NW, offset, FLOOR_Z, color);
        }
        if (self.loc + V2(0, -1)).terrain().is_hole() {
            self.draw_tile(ctx, BLOCK_NE, offset, FLOOR_Z, color);
        }
    }

    fn draw_floor(&'a self, ctx: &mut Canvas, idx: usize, offset: V2<f32>, z: f32, color: &Rgb) {
        self.draw_tile(ctx, idx, offset, z, color);
        self.floor_edges(ctx, offset, color);
    }

    fn draw_tile2(&'a self, ctx: &mut Canvas, idx: usize, offset: V2<f32>, z: f32,
                  color: &Rgb, back_color: &Rgb) {
        let (mut color, mut back_color) = match self.fov {
            // XXX: Special case for the solid-black objects that are used to
            // block out stuff to not get recolored. Don't use total black as
            // an actual object color, have something like #010101 instead.
            Some(FovStatus::Remembered) if *color != BLACK => (BLACK, Rgb::new(0x33, 0x00, 0x22)),
            _ => (*color, *back_color),
        };
        if self.fov == Some(FovStatus::Seen) {
            color = self.light.apply(&color);
            back_color = self.light.apply(&back_color);
            if self.depth != 0 && color != BLACK {
                back_color = Rgb::new(
                    0x20 * -self.depth as u8,
                    0x20 * -self.depth as u8,
                    0x20 * -self.depth as u8);
                color = Rgb::new(
                    (color.r as f32 * 0.5) as u8,
                    (color.g as f32 * 0.5) as u8,
                    (color.b as f32 * 0.5) as u8);
            }
        }
        let z = z + self.depth as f32 * DEPTH_Z_MODIFIER;
        if self.depth != 0 && self.fov != Some(FovStatus::Seen) { return; }

        let offset = offset + level_z_to_view(self.depth).map(|x| x as f32);
        ctx.draw_image(tilecache::get(idx), offset, z, &color, &back_color);
    }

    fn draw_cell(&'a self, ctx: &mut Canvas, offset: V2<f32>) {
        if !self.loc.terrain().is_hole() {
            self.draw_terrain(ctx, offset);
        }

        if self.fov == Some(FovStatus::Seen) && self.depth == 0 {
            // Sort mobs on top of items for drawing.
            let mut es = self.loc.entities();
            es.sort_by(|a, b| a.is_mob().cmp(&b.is_mob()));
            for e in es.iter() {
                self.draw_entity(ctx, offset, e);
            }
        }
    }

    fn draw_terrain(&'a self, ctx: &mut Canvas, offset: V2<f32>) {
        let k = Kernel::new(|loc| loc.terrain(), self.loc);
        match k.center {
            TerrainType::Void => {
                self.draw_tile(ctx, BLANK_FLOOR, offset, FLOOR_Z, &BLACK);
            },
            TerrainType::Water => {
                self.draw_floor(ctx, WATER, offset, FLOOR_Z, &ROYALBLUE);
            },
            TerrainType::Shallows => {
                self.draw_floor(ctx, SHALLOWS, offset, FLOOR_Z, &CORNFLOWERBLUE);
            },
            TerrainType::Magma => {
                self.draw_tile2(ctx, MAGMA, offset, FLOOR_Z, &DARKRED, &YELLOW);
                self.floor_edges(ctx, offset, &YELLOW);
            },
            TerrainType::Tree => {
                // A two-toner, with floor, using two z-layers
                self.draw_floor(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, TREE_TRUNK, offset, BLOCK_Z, &DARKORANGE);
                self.draw_tile(ctx, TREE_FOLIAGE, offset, BLOCK_Z, &PURPLE);
            },
            TerrainType::Floor => {
                self.draw_floor(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
            },
            TerrainType::Chasm => {
                self.draw_tile(ctx, CHASM, offset, FLOOR_Z, &DARKSLATEGRAY);
            },
            TerrainType::Grass => {
                self.draw_floor(ctx, FLOOR, offset, FLOOR_Z, &DARKSLATEBLUE);
            },
            TerrainType::Grass2 => {
                self.draw_floor(ctx, GRASS, offset, FLOOR_Z, &DARKSLATEBLUE);
            },
            TerrainType::Rock => {
                blockform(self, ctx, &k, offset, BLOCK, &SILVER);
            }
            TerrainType::Wall => {
                self.draw_floor(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                wallform(self, ctx, &k, offset, WALL, &LIGHTSLATEGRAY, true);
            },
            TerrainType::Stalagmite => {
                self.draw_floor(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, STALAGMITE, offset, BLOCK_Z, &DARKGOLDENROD);
            },
            TerrainType::Window => {
                self.draw_floor(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                wallform(self, ctx, &k, offset, WINDOW, &LIGHTSLATEGRAY, false);
            },
            TerrainType::Door => {
                self.draw_floor(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                wallform(self, ctx, &k, offset, DOOR, &LIGHTSLATEGRAY, true);
                wallform(self, ctx, &k, offset, DOOR + 4, &DARKTURQUOISE, false);
            },
            TerrainType::OpenDoor => {
                self.draw_floor(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                wallform(self, ctx, &k, offset, DOOR, &LIGHTSLATEGRAY, true);
            },
            TerrainType::Table => {
                self.draw_floor(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, TABLE, offset, BLOCK_Z, &SILVER);
            },
            TerrainType::Barrel => {
                self.draw_floor(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, BARREL, offset, BLOCK_Z, &OLIVEDRAB);
            },
            TerrainType::Stone => {
                self.draw_floor(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, STONE, offset, BLOCK_Z, &SLATEGRAY);
            },
            TerrainType::DeadTree => {
                self.draw_floor(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, TREE_TRUNK, offset, BLOCK_Z, &SADDLEBROWN);
            },
            TerrainType::TallGrass => {
                self.draw_tile(ctx, TALLGRASS, offset, BLOCK_Z, &MEDIUMORCHID);
            },
            TerrainType::CraterN => {
                self.draw_floor(ctx, CRATER_N, offset, FLOOR_Z, &SLATEGRAY);
            },
            TerrainType::CraterNE => {
                self.draw_floor(ctx, CRATER_NE, offset, FLOOR_Z, &SLATEGRAY);
            },
            TerrainType::CraterSE => {
                self.draw_floor(ctx, CRATER_SE, offset, FLOOR_Z, &SLATEGRAY);
            },
            TerrainType::CraterS => {
                self.draw_floor(ctx, CRATER_S, offset, FLOOR_Z, &SLATEGRAY);
            },
            TerrainType::CraterSW => {
                self.draw_floor(ctx, CRATER_SW, offset, FLOOR_Z, &SLATEGRAY);
            },
            TerrainType::CraterNW => {
                self.draw_floor(ctx, CRATER_NW, offset, FLOOR_Z, &SLATEGRAY);
            },
            TerrainType::Crater => {
                self.draw_floor(ctx, CRATER, offset, FLOOR_Z, &SLATEGRAY);
            },
            TerrainType::Pod => {
                self.draw_floor(ctx, FLOOR, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_floor(ctx, POD, offset, BLOCK_Z, &DARKCYAN);
            },
        }

        fn blockform(c: &CellDrawable, ctx: &mut Canvas, k: &Kernel<TerrainType>, mut offset: V2<f32>, idx: usize, color: &Rgb) {
            if c.depth != 0 {
                c.draw_tile(ctx, idx, offset, BLOCK_Z, color);
                // Double blockforms in sub-levels.
                offset = offset + V2(0, -PIXEL_UNIT/2).map(|x| x as f32);
            }
            c.draw_tile(ctx, BLOCK_DARK, offset, BLOCK_Z, &BLACK);
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

        fn wallform(c: &CellDrawable, ctx: &mut Canvas, k: &Kernel<TerrainType>, offset: V2<f32>, idx: usize, color: &Rgb, opaque: bool) {
            let (left_wall, right_wall, block) = wall_flags_lrb(k);
            if block {
                if opaque {
                    c.draw_tile(ctx, CUBE, offset, BLOCK_Z, &BLACK);
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
                /*
                let idx = if k.n.is_wall() && (!k.nw.is_wall() || !k.ne.is_wall()) {
                    // TODO: Walltile-specific XY-walls
                    XYWALL
                } else {
                    idx + 3
                };
                c.draw_tile(ctx, idx + 3, offset, BLOCK_Z, color);
                */
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

    fn draw_entity(&'a self, ctx: &mut Canvas, offset: V2<f32>, entity: &Entity) {
        // SPECIAL CASE: The serpent mob has an extra mound element that
        // doesn't bob along with the main body.
        static SERPENT_ICON: usize = 94;

        let body_pos =
            if entity.is_bobbing() {
                offset + *(timing::cycle_anim(
                        0.3f64,
                        &[V2(0.0, 0.0), V2(0.0, -1.0)]))
            } else { offset };

        if let Some((icon, mut color)) = entity.get_icon() {
            let mut back_color = BLACK;

            // Damage blink animation.
            if let Some(&t) = self.damage_timers.get(entity) {
                if t % 2 == 0 {
                    color = WHITE;
                    back_color = WHITE;
                } else {
                    color = BLACK;
                    back_color = BLACK;
                }
            }

            if entity.is_item() {
                // Blink pickups intermittently to draw attention.
                if timing::spike(1.5, 0.1) {
                    color = WHITE;
                    back_color = WHITE;
                }
            }

            if icon == SERPENT_ICON {
                // Body
                self.draw_tile2(ctx, icon, body_pos, BLOCK_Z, &color, &back_color);
                // Ground mound, doesn't bob.
                self.draw_tile2(ctx, icon + 1, offset, BLOCK_Z, &color, &back_color);
                return;
            }

            self.draw_tile2(ctx, icon, body_pos, BLOCK_Z, &color, &back_color);
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
    pub fn new<F>(get: F, loc: Location) -> Kernel<C>
        where F: Fn(Location) -> C {
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
