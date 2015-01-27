use std::collections::HashMap;
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
    // Draw stuff at most this deep.
    static MIN_DRAWN_DEPTH: i32 = 8;

    for depth in -1..(MIN_DRAWN_DEPTH) {
        let mut hole_seen = false;
        for pt in cells_on_screen() {
            // Displace stuff deeper down to compensate for the projection
            // that shifts lower z-levels off-screen.
            let pt = pt + V2(depth, depth);
            let screen_pos = chart_to_screen(pt);
            let loc = *chart + pt;
            let depth_loc = Location { z: loc.z + depth as i8, ..loc };
            hole_seen |= depth_loc.terrain().is_space();
            // XXX: Grab FOV and light from zero z layer. Not sure what the
            // really right approach here is.
            let cell_drawable = CellDrawable::new(
                depth_loc, depth, loc.fov_status(), loc.light(), damage_timers);
            cell_drawable.draw(ctx, screen_pos);
        }
        // Don't draw the lower level unless there was at least one hole.
        if depth >= 0 && !hole_seen { return; }
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
        self.draw_tile2(ctx, idx, offset, z, self.depth, color, &BLACK);
    }

    /// Draw edge lines to floor tile if there are chasm tiles to the back.
    fn floor_edges(&'a self, ctx: &mut Canvas, offset: V2<f32>, color: &Rgb) {
        self.draw_tile(ctx, FLOOR_FRONT, offset, FLOOR_Z, color);

        // Shift edge offset from block top level to floor level.
        let offset = offset + V2(0, PIXEL_UNIT / 2).map(|x| x as f32);

        if (self.loc + V2(-1, -1)).terrain().is_space() {
            self.draw_tile(ctx, BLOCK_N, offset, FLOOR_Z, color);
        }
        if (self.loc + V2(-1, 0)).terrain().is_space() {
            self.draw_tile(ctx, BLOCK_NW, offset, FLOOR_Z, color);
        }
        if (self.loc + V2(0, -1)).terrain().is_space() {
            self.draw_tile(ctx, BLOCK_NE, offset, FLOOR_Z, color);
        }
    }

    fn draw_floor(&'a self, ctx: &mut Canvas, idx: usize, offset: V2<f32>, color: &Rgb, edges: bool) {
        // Gray out the back color for lower-depth floor to highlight that
        // it's not real floor.
        let depth = if self.depth > 0 { self.depth as u8 } else { 0 };
        let back_color = Rgb::new(
            0x10 * depth,
            0x10 * depth,
            0x10 * depth);
        self.draw_tile2(ctx, idx, offset, FLOOR_Z, self.depth, color, &back_color);
        if edges {
            self.floor_edges(ctx, offset, color);
        }
    }

    fn draw_tile2(&'a self, ctx: &mut Canvas, idx: usize, offset: V2<f32>, z: f32,
                  depth: i32, color: &Rgb, back_color: &Rgb) {
        let map_color = if depth == 0 {
            Rgb::new(0x33, 0x22, 0x00) } else { Rgb::new(0x22, 0x11, 0x00) };

        let (mut color, mut back_color) = match self.fov {
            // XXX: Special case for the solid-black objects that are used to
            // block out stuff to not get recolored. Don't use total black as
            // an actual object color, have something like #010101 instead.
            Some(FovStatus::Remembered) if *color != BLACK => (BLACK, map_color),
            _ => (*color, *back_color),
        };
        if self.fov == Some(FovStatus::Seen) {
            color = self.light.apply(&color);
            back_color = self.light.apply(&back_color);
            if depth > 0 && color != BLACK {
                color = Rgb::new(
                    (color.r as f32 * (1.0 - (depth as f32) / 8.0)) as u8,
                    (color.g as f32 * (1.0 - (depth as f32) / 8.0)) as u8,
                    (color.b as f32 * (1.0 - (depth as f32) / 8.0)) as u8);
            }
        }
        let z = z + self.depth as f32 * DEPTH_Z_MODIFIER;

        let offset = offset + level_z_to_view(depth).map(|x| x as f32);
        ctx.draw_image(tilecache::get(idx), offset, z, &color, &back_color);
    }

    fn draw_cell(&'a self, ctx: &mut Canvas, offset: V2<f32>) {
        if !self.loc.terrain().is_space() {
            if self.loc.terrain().is_block() {
                self.draw_block(ctx, offset);
            } else if self.loc.terrain().is_wall() {
                self.draw_wall(ctx, offset);
            } else {
                self.draw_terrain(ctx, offset);
            }
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

    fn draw_block(&'a self, ctx: &mut Canvas, offset: V2<f32>) {
        let k = Kernel::new(|loc| loc.terrain(), self.loc);

        let (wall_idx, wall_color, top_idx, top_color) = match k.center {
            TerrainType::Rock => (ROCK_BLOCK, &DARKGOLDENROD, FLOOR_BLOCK_TOP, &SLATEGRAY),
            _ => panic!("Unhandled hull type {:?}", k.center)
        };

        let draw_top =
            // Draw the top tile on all exposed blocks deeper than the
            // current Z-level. (Blocks at z+1 form the floor for the
            // current plane.).
            (!k.up.is_hull() && self.depth > 0) ||
            // Also draw ramp tops, but only if there's empty space (no
            // hull and no prop) on top.
            (k.up.is_space() && self.depth == 0);
        let draw_left = k.nw.is_hull() || k.sw.is_hull();
        let draw_right = k.ne.is_hull() || k.se.is_hull();

        if !draw_left && !draw_right {
            // Singleton block.
            self.draw_tile(ctx, wall_idx + 2, offset, BLOCK_Z, wall_color);
            self.draw_tile(ctx, wall_idx + 3, offset, BLOCK_Z, wall_color);

            // Draw the singleton top.
            if draw_top {
                // Mess with depth for top stuff to get it colored with the
                // higher level's lighting.
                self.draw_tile2(ctx, top_idx + 2, offset,
                    BLOCK_Z, self.depth - 1, top_color, &BLACK);
                self.draw_tile2(ctx, top_idx + 3, offset,
                    BLOCK_Z, self.depth - 1, top_color, &BLACK);
            }
            return;
        }

        // We know that there's at least one hull neighbor now. Either the
        // left or the right side can be clipped off if that side has no
        // neighbors.

        // Left half.
        if draw_left {
            if !k.nw.is_hull() && k.below_nw.is_hull() {
                self.draw_tile(ctx, BACK_EDGE, offset, BLOCK_Z, wall_color);
            }

            if !k.sw.is_hull() {
                self.draw_tile(ctx, wall_idx, offset, BLOCK_Z, wall_color);
            }

            if draw_top {
                self.draw_tile2(ctx, top_idx, offset,
                    BLOCK_Z, self.depth - 1, top_color, &BLACK);

                // Ground drops away behind, draw the boundary line.
                if !k.nw.is_hull() {
                    self.draw_tile2(ctx, BACK_EDGE, offset, BLOCK_Z,
                        self.depth - 1, top_color, &BLACK);
                }
            }

            if !draw_right {
                if !k.e.is_hull() && k.below_e.is_hull() {
                    self.draw_tile(ctx, SIDE_EDGE, offset,
                        BLOCK_Z, wall_color);
                }

                if draw_top {
                    self.draw_tile2(ctx, SIDE_EDGE, offset, BLOCK_Z,
                        self.depth - 1, top_color, &BLACK);
                }
            }
        }

        // Right half
        if draw_right {
            if !k.ne.is_hull() && k.below_ne.is_hull() {
                self.draw_tile(ctx, BACK_EDGE + 1, offset, BLOCK_Z, wall_color);
            }

            if !k.se.is_hull() {
                self.draw_tile(ctx, wall_idx + 1, offset, BLOCK_Z, wall_color);
            }

            if draw_top {
                self.draw_tile2(ctx, top_idx + 1, offset,
                    BLOCK_Z, self.depth - 1, top_color, &BLACK);
                // Ground drops away behind, draw the boundary line.
                if !k.ne.is_hull() {
                    self.draw_tile2(ctx, BACK_EDGE + 1, offset, BLOCK_Z,
                        self.depth - 1, top_color, &BLACK);
                }
            }

            if !draw_left {
                if !k.w.is_hull() && k.below_w.is_hull() {
                    self.draw_tile(ctx, SIDE_EDGE + 1, offset,
                        BLOCK_Z, wall_color);
                }

                if draw_top {
                    self.draw_tile2(ctx, SIDE_EDGE + 1, offset,
                        BLOCK_Z, self.depth - 1, top_color, &BLACK);
                }
            }
        }
    }

    fn draw_wall(&'a self, ctx: &mut Canvas, offset: V2<f32>) {
        let k = Kernel::new(|loc| loc.terrain(), self.loc);

        let (wall_idx, wall_color, top_idx, top_color) = match k.center {
            TerrainType::Wall => (WALL_BLOCK, &LIGHTSLATEGRAY, FLOOR_BLOCK_TOP, &SLATEGRAY),
            TerrainType::Window => (WINDOW_BLOCK, &LIGHTSLATEGRAY, FLOOR_BLOCK_TOP, &SLATEGRAY),
            _ => panic!("Unhandled hull type {:?}", k.center)
        };

        let draw_top =
            // Draw the top tile on all exposed blocks deeper than the
            // current Z-level. (Blocks at z+1 form the floor for the
            // current plane.).
            (!k.up.is_hull() && self.depth > 0) ||
            // Also draw ramp tops, but only if there's empty space (no
            // hull and no prop) on top.
            (k.up.is_space() && self.depth == 0);
        let extend_left = k.nw.is_hull();
        let extend_right = k.ne.is_hull();
        let is_thick = extend_left && extend_right && k.n.is_hull();

        // Left half.
        if !k.nw.is_hull() && k.below_nw.is_hull() {
            self.draw_tile(ctx, BACK_EDGE, offset, BLOCK_Z, wall_color);
        }

        if !k.sw.is_hull() {
            let idx = if extend_left { wall_idx } else { wall_idx + 2 };
            self.draw_tile(ctx, idx, offset, BLOCK_Z, wall_color);
        }

        if draw_top {
            if is_thick {
                self.draw_tile2(ctx, top_idx, offset,
                    BLOCK_Z, self.depth - 1, top_color, &BLACK);

                // Ground drops away behind, draw the boundary line.
                if !k.nw.is_hull() {
                    self.draw_tile2(ctx, BACK_EDGE, offset, BLOCK_Z,
                        self.depth - 1, top_color, &BLACK);
                }
            } else if extend_left {
                self.draw_tile2(ctx, WALL_BLOCK_TOP, offset,
                    BLOCK_Z, self.depth - 1, top_color, &BLACK);
            } else {
                self.draw_tile2(ctx, WALL_BLOCK_TOP + 2, offset,
                    BLOCK_Z, self.depth - 1, top_color, &BLACK);
            }
        }

        // Right half.
        if !k.ne.is_hull() && k.below_ne.is_hull() {
            self.draw_tile(ctx, BACK_EDGE + 1, offset, BLOCK_Z, wall_color);
        }

        if !k.se.is_hull() {
            let idx = if extend_right { wall_idx + 1 } else { wall_idx + 3 };
            self.draw_tile(ctx, idx, offset, BLOCK_Z, wall_color);
        }

        if draw_top {
            if is_thick {
                self.draw_tile2(ctx, top_idx + 1, offset,
                    BLOCK_Z, self.depth - 1, top_color, &BLACK);

                // Ground drops away behind, draw the boundary line.
                if !k.ne.is_hull() {
                    self.draw_tile2(ctx, BACK_EDGE + 1, offset, BLOCK_Z,
                        self.depth - 1, top_color, &BLACK);
                }
            } else if extend_right {
                self.draw_tile2(ctx, WALL_BLOCK_TOP + 1, offset,
                    BLOCK_Z, self.depth - 1, top_color, &BLACK);
            } else {
                self.draw_tile2(ctx, WALL_BLOCK_TOP + 3, offset,
                    BLOCK_Z, self.depth - 1, top_color, &BLACK);
            }
        }
    }

    fn draw_terrain(&'a self, ctx: &mut Canvas, offset: V2<f32>) {
        let k = Kernel::new(|loc| loc.terrain(), self.loc);

        match k.center {
            TerrainType::Space => {
                //self.draw_tile(ctx, BLANK_FLOOR, offset, FLOOR_Z, &BLACK);
            },
            TerrainType::Water => {
                self.draw_floor(ctx, WATER, offset, &ROYALBLUE, true);
            },
            TerrainType::Shallows => {
                self.draw_floor(ctx, SHALLOWS, offset, &CORNFLOWERBLUE, true);
            },
            TerrainType::Magma => {
                self.draw_tile2(ctx, MAGMA, offset, FLOOR_Z, self.depth, &DARKRED, &YELLOW);
                self.floor_edges(ctx, offset, &YELLOW);
            },
            TerrainType::Tree => {
                self.draw_tile(ctx, TREE_TRUNK, offset, BLOCK_Z, &SADDLEBROWN);
                self.draw_tile(ctx, TREE_FOLIAGE, offset, BLOCK_Z, &GREEN);
            },
            TerrainType::Floor => {
                //self.draw_floor(ctx, FLOOR, offset, &SLATEGRAY, true);
                self.draw_tile(ctx, FLOOR_BLOCK_TOP, offset, FLOOR_Z, &SLATEGRAY);
                self.draw_tile(ctx, FLOOR_BLOCK_TOP + 1, offset, FLOOR_Z, &SLATEGRAY);
            },
            TerrainType::Grass => {
                self.draw_floor(ctx, FLOOR, offset, &DARKGREEN, true);
            },
            TerrainType::Grass2 => {
                self.draw_floor(ctx, GRASS, offset, &DARKGREEN, true);
            },
            TerrainType::Downstairs => {
                self.draw_floor(ctx, FLOOR, offset, &SLATEGRAY, true);
                self.draw_tile(ctx, DOWNSTAIRS, offset, BLOCK_Z, &SLATEGRAY);
            },
            TerrainType::Rock => {
                panic!("Hull terrain in regular draw");
            }
            TerrainType::Wall => {
                // TODO: New-style wall formatting.
                panic!("Hull terrain in regular draw");
            },
            TerrainType::RockWall => {
                wallfloor(self, ctx, &k, offset);
                wallform(self, ctx, &k, offset, ROCKWALL, &LIGHTSLATEGRAY, true);
            },
            TerrainType::Fence => {
                // The floor type beneath the fence tile is visible, make it grass
                // if there's grass behind the fence. Otherwise make it regular
                // floor.
                let front_of_hole = k.nw.is_space() || k.n.is_space() || k.ne.is_space();
                if !front_of_hole {
                    if k.n == TerrainType::Grass || k.ne == TerrainType::Grass || k.nw == TerrainType::Grass {
                        self.draw_floor(ctx, GRASS, offset, &DARKGREEN, true);
                    } else {
                        self.draw_floor(ctx, FLOOR, offset, &SLATEGRAY, true);
                    }
                }
                wallform(self, ctx, &k, offset, FENCE, &DARKGOLDENROD, false);
            },
            TerrainType::Bars => {
                wallfloor(self, ctx, &k, offset);
                wallform(self, ctx, &k, offset, BARS, &GAINSBORO, false);
            },
            TerrainType::Stalagmite => {
                self.draw_floor(ctx, FLOOR, offset, &SLATEGRAY, true);
                self.draw_tile(ctx, STALAGMITE, offset, BLOCK_Z, &DARKGOLDENROD);
            },
            TerrainType::Window => {
                wallfloor(self, ctx, &k, offset);
                wallform(self, ctx, &k, offset, WINDOW, &LIGHTSLATEGRAY, false);
            },
            TerrainType::Door => {
                wallfloor(self, ctx, &k, offset);
                wallform(self, ctx, &k, offset, DOOR, &LIGHTSLATEGRAY, true);
                wallform(self, ctx, &k, offset, DOOR + 4, &SADDLEBROWN, false);
            },
            TerrainType::OpenDoor => {
                wallfloor(self, ctx, &k, offset);
                wallform(self, ctx, &k, offset, DOOR, &LIGHTSLATEGRAY, true);
            },
            TerrainType::Table => {
                self.draw_floor(ctx, FLOOR, offset, &SLATEGRAY, true);
                self.draw_tile(ctx, TABLE, offset, BLOCK_Z, &DARKGOLDENROD);
            },
            TerrainType::Fountain => {
                self.draw_floor(ctx, FLOOR, offset, &SLATEGRAY, true);
                self.draw_tile(ctx, FOUNTAIN, offset, BLOCK_Z, &GAINSBORO);
            },
            TerrainType::Altar => {
                self.draw_floor(ctx, FLOOR, offset, &SLATEGRAY, true);
                self.draw_tile(ctx, ALTAR, offset, BLOCK_Z, &GAINSBORO);
            },
            TerrainType::Barrel => {
                self.draw_floor(ctx, FLOOR, offset, &SLATEGRAY, true);
                self.draw_tile(ctx, BARREL, offset, BLOCK_Z, &DARKGOLDENROD);
            },
            TerrainType::Grave => {
                self.draw_floor(ctx, FLOOR, offset, &SLATEGRAY, true);
                self.draw_tile(ctx, GRAVE, offset, BLOCK_Z, &SLATEGRAY);
            },
            TerrainType::Stone => {
                self.draw_floor(ctx, FLOOR, offset, &SLATEGRAY, true);
                self.draw_tile(ctx, STONE, offset, BLOCK_Z, &SLATEGRAY);
            },
            TerrainType::Menhir => {
                self.draw_floor(ctx, FLOOR, offset, &SLATEGRAY, true);
                self.draw_tile(ctx, MENHIR, offset, BLOCK_Z, &SLATEGRAY);
            },
            TerrainType::DeadTree => {
                self.draw_floor(ctx, FLOOR, offset, &SLATEGRAY, true);
                self.draw_tile(ctx, TREE_TRUNK, offset, BLOCK_Z, &SADDLEBROWN);
            },
            TerrainType::TallGrass => {
                self.draw_tile(ctx, TALLGRASS, offset, BLOCK_Z, &GOLD);
            },
            TerrainType::Battlement => {
                wallfloor(self, ctx, &k, offset);
                wallform(self, ctx, &k, offset, BATTLEMENT, &LIGHTSLATEGRAY, true);
            },
        }

        fn wallfloor(c: &CellDrawable, ctx: &mut Canvas, k: &Kernel<TerrainType>, offset: V2<f32>) {
            // In front of a hole, no floor.
            if k.nw.is_space() || k.n.is_space() || k.ne.is_space() { return; }
            c.draw_floor(ctx, FLOOR, offset, &SLATEGRAY, false);
        }

        fn wallform(c: &CellDrawable, ctx: &mut Canvas, k: &Kernel<TerrainType>, offset: V2<f32>, idx: usize, color: &Rgb, opaque: bool) {
            let (left_wall, right_wall, block) = wall_flags_lrb(k);
            // HACK: You can walk on top of battlements, so place them between
            // the regular block z and floor z so that they show below the
            // entity sprites.
            let z = if idx == BATTLEMENT { BLOCK_Z + 0.0005 } else { BLOCK_Z };
            if block {
                if opaque {
                    c.draw_tile(ctx, CUBE, offset, z, &BLACK);
                } else {
                    c.draw_tile(ctx, idx + 2, offset, z, color);
                    return;
                }
            }
            if left_wall && right_wall {
                c.draw_tile(ctx, idx + 2, offset, z, color);
            } else if left_wall {
                c.draw_tile(ctx, idx, offset, z, color);
            } else if right_wall {
                c.draw_tile(ctx, idx + 1, offset, z, color);
            } else if !block || !k.s.is_wall() {
                // NB: This branch has some actual local kernel logic not
                // handled by wall_flags_lrb.
                let idx = if k.n.is_wall() && (!k.nw.is_wall() || !k.ne.is_wall()) {
                    // TODO: Walltile-specific XY-walls
                    XYWALL
                } else {
                    idx + 3
                };
                c.draw_tile(ctx, idx, offset, z, color);
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
            // Damage blink animation.
            if let Some(&t) = self.damage_timers.get(entity) {
                color = if t % 2 == 0 { WHITE } else { BLACK };
            }

            if icon == SERPENT_ICON {
                // Body
                self.draw_tile(ctx, icon, body_pos, BLOCK_Z, &color);
                // Ground mound, doesn't bob.
                self.draw_tile(ctx, icon + 1, offset, BLOCK_Z, &color);
            } else {
                self.draw_tile(ctx, icon, body_pos, BLOCK_Z, &color);
            }
        }
    }
}

/// 3x3 grid of terrain cells. Use this as the input for terrain tile
/// computation, which will need to consider the immediate vicinity of cells.
struct Kernel<C> {
    n: C,
    ne: C,
    below_ne: C,
    e: C,
    below_e: C,
    nw: C,
    below_nw: C,
    center: C,
    se: C,
    w: C,
    below_w: C,
    sw: C,
    s: C,
    up: C,
}

impl<C: Clone> Kernel<C> {
    pub fn new<F>(get: F, loc: Location) -> Kernel<C>
        where F: Fn(Location) -> C {
        Kernel {
            n: get(loc + V2(-1, -1)),
            ne: get(loc + V2(0, -1)),
            below_ne: get(Location { z: loc.z + 1, ..loc + V2(0, -1) }),
            e: get(loc + V2(1, -1)),
            below_e: get(Location { z: loc.z + 1, ..loc + V2(1, -1) }),
            nw: get(loc + V2(-1, 0)),
            below_nw: get(Location { z: loc.z + 1, ..loc + V2(-1, 0) }),
            center: get(loc),
            se: get(loc + V2(1, 0)),
            w: get(loc + V2(-1, 1)),
            below_w: get(Location { z: loc.z + 1, ..loc + V2(-1, 1) }),
            sw: get(loc + V2(0, 1)),
            s: get(loc + V2(1, 1)),
            up: get(Location { z: loc.z - 1, ..loc }),
        }
    }
}
