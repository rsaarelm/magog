use std::collections::HashMap;
use util::{V2, Rgb, Rgba, timing};
use util::color::*;
use backend::{Canvas, CanvasUtil};
use world::TerrainType;
use world::{Location, Chart};
use world::{FovStatus};
use world::{Entity};
use world::{Light};
use viewutil::{chart_to_screen, cells_on_screen, level_z_to_view};
use viewutil::{TILE_Z, DEPTH_Z_MODIFIER};
use drawable::{Drawable};
use tilecache;
use tilecache::tile::*;

pub fn draw_world<C: Chart+Copy>(chart: &C, ctx: &mut Canvas, damage_timers: &HashMap<Entity, u32>) {
    // Draw stuff at most this deep.
    static MAX_DRAWN_DEPTH: i32 = 8;

    for depth in -1..(MAX_DRAWN_DEPTH) {
        // TODO: Special draw for the above-ground layer (just ramps)
        if depth == -1 { continue; }

        // Automatically go through from depth -1, beyond that must see a hole
        // to bother continuing.
        let mut hole_seen = depth < 0;
        for pt in cells_on_screen() {
            // Displace stuff deeper down to compensate for the projection
            // that shifts lower z-levels off-screen.
            let pt = pt + V2(depth, depth);
            let screen_pos = chart_to_screen(pt);
            let loc = *chart + pt;
            let depth_loc = Location { z: loc.z + depth as i8, ..loc };
            hole_seen |= !depth_loc.below().terrain().is_block();

            // TODO: Light for lower levels. Currently just grabbing it from
            // the z = 0 layer.
            let light = loc.light();

            // TODO: More principled FOV status setting than just taking the
            // status of the z=0 cell.
            let fov = loc.fov_status();

            let cell_drawable = CellDrawable::new(
                depth_loc, depth, fov, light, damage_timers);
            cell_drawable.draw(ctx, screen_pos);
        }
        // Don't draw the lower level unless there was at least one hole.
        if !hole_seen { return; }
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
                self.draw_tile(ctx, BLANK_FLOOR, offset, &BLACK);
            }
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

    fn draw_tile(&'a self, ctx: &mut Canvas, idx: usize, offset: V2<f32>, color: &Rgb) {
        self.draw_tile2(ctx, idx, offset, color, &BLACK);
    }

    /// Draw edge lines to floor tile if there are chasm tiles to the back.
    fn floor_edges(&'a self, ctx: &mut Canvas, offset: V2<f32>, color: &Rgb) {
        if (self.loc + V2(-1, -1)).below().terrain().is_space() {
            self.draw_tile(ctx, BLOCK_N, offset, color);
        }
        if (self.loc + V2(-1, 0)).below().terrain().is_space() {
            self.draw_tile(ctx, BLOCK_NW, offset, color);
        }
        if (self.loc + V2(0, -1)).below().terrain().is_space() {
            self.draw_tile(ctx, BLOCK_NE, offset, color);
        }
    }

    fn draw_tile2(&'a self, ctx: &mut Canvas, idx: usize, offset: V2<f32>,
                  color: &Rgb, back_color: &Rgb) {
        let map_color = if self.depth == 0 {
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
            if self.depth != 0 && color != BLACK {
                color = Rgb::new(
                    (color.r as f32 * 0.5) as u8,
                    (color.g as f32 * 0.5) as u8,
                    (color.b as f32 * 0.5) as u8);
            }
        }
        let z = TILE_Z + self.depth as f32 * DEPTH_Z_MODIFIER;

        let offset = offset + level_z_to_view(self.depth).map(|x| x as f32);

        if self.depth == -1 {
            // Transparent up-stuff.
            let color = Rgba { r: color.r, g: color.g, b: color.b, a: 0x80 };
            let back_color = Rgba { r: back_color.r, g: back_color.g, b: back_color.b, a: 0x80 };
            ctx.draw_image(tilecache::get(idx), offset, z, &color, &back_color);
        } else {
            ctx.draw_image(tilecache::get(idx), offset, z, &color, &back_color);
        }
    }

    fn draw_cell(&'a self, ctx: &mut Canvas, offset: V2<f32>) {
        self.draw_floor(ctx, offset);
        self.draw_terrain(ctx, offset);

        if self.fov == Some(FovStatus::Seen) && self.depth == 0 {
            // Sort mobs on top of items for drawing.
            let mut es = self.loc.entities();
            es.sort_by(|a, b| a.is_mob().cmp(&b.is_mob()));
            for e in es.iter() {
                self.draw_entity(ctx, offset, e);
            }
        }
    }

    fn draw_floor(&'a self, ctx: &mut Canvas, offset: V2<f32>) {
        let t = self.loc.below().terrain();
        if self.depth == -1 {
            // TODO
            // Special case.
        }
        if t.is_block() {
            let (idx, color, color2) = match t {
                TerrainType::Rock => (FLOOR, &SLATEGRAY, &BLACK),
                TerrainType::Water => (WATER, &ROYALBLUE, &BLACK),
                TerrainType::Shallows => (SHALLOWS, &CORNFLOWERBLUE, &BLACK),
                TerrainType::Magma => (MAGMA, &DARKRED, &YELLOW),
                TerrainType::Grass => (FLOOR, &DARKGREEN, &BLACK),
                TerrainType::Grass2 => (GRASS, &DARKGREEN, &BLACK),
                _ => panic!("Unknown block type {:?}", t),
            };

            self.draw_tile2(ctx, idx, offset, color, color2);
            self.floor_edges(ctx, offset, color);
        }
        if t.is_wall() && !self.is_exposed_wall(self.loc.below()) {
            self.draw_tile(ctx, FLOOR, offset, &SLATEGRAY)
        }

        // TODO: Draw wall top at z == -1
    }

    fn draw_terrain(&'a self, ctx: &mut Canvas, offset: V2<f32>) {
        let k = Kernel::new(|loc| loc.terrain(), self.loc);

        match k.center {
            TerrainType::Space => {},
            TerrainType::Water => {
                //self.draw_floor(ctx, WATER, offset, &ROYALBLUE, true);
            },
            TerrainType::Shallows => {
                //self.draw_floor(ctx, SHALLOWS, offset, &CORNFLOWERBLUE, true);
            },
            TerrainType::Magma => {
                //self.draw_tile2(ctx, MAGMA, offset, &DARKRED, &YELLOW);
                //self.floor_edges(ctx, offset, &YELLOW);
            },
            TerrainType::Tree => {
                // A two-toner, with floor, using two z-layers
                //self.draw_floor(ctx, FLOOR, offset, &SLATEGRAY, true);
                self.draw_tile(ctx, TREE_TRUNK, offset, &SADDLEBROWN);
                self.draw_tile(ctx, TREE_FOLIAGE, offset, &GREEN);
            },
            TerrainType::Grass => {
                //self.draw_floor(ctx, FLOOR, offset, &DARKGREEN, true);
            },
            TerrainType::Grass2 => {
                //self.draw_floor(ctx, GRASS, offset, &DARKGREEN, true);
            },
            TerrainType::Downstairs => {
                //self.draw_floor(ctx, FLOOR, offset, &SLATEGRAY, true);
                self.draw_tile(ctx, DOWNSTAIRS, offset, &SLATEGRAY);
            },
            TerrainType::Rock => {
                self.draw_block(ctx, &k, offset, ROCK, &DARKGOLDENROD);
            }
            TerrainType::Wall => {
                self.draw_wall(ctx, &k, offset, WALL, &LIGHTSLATEGRAY);
            },
            TerrainType::RockWall => {
                self.draw_wall(ctx, &k, offset, ROCKWALL, &LIGHTSLATEGRAY);
            },
            TerrainType::Fence => {
                self.draw_wall(ctx, &k, offset, BARS, &DARKGOLDENROD);
            },
            TerrainType::Bars => {
                self.draw_wall(ctx, &k, offset, BARS, &GAINSBORO);
            },
            TerrainType::Stalagmite => {
                self.draw_tile(ctx, STALAGMITE, offset, &DARKGOLDENROD);
            },
            TerrainType::Window => {
                self.draw_wall(ctx, &k, offset, WINDOW, &LIGHTSLATEGRAY);
            },
            TerrainType::Door => {
                self.draw_wall(ctx, &k, offset, DOOR, &LIGHTSLATEGRAY);
                self.draw_wall(ctx, &k, offset, DOOR + 6, &SADDLEBROWN);
            },
            TerrainType::OpenDoor => {
                self.draw_wall(ctx, &k, offset, DOOR, &LIGHTSLATEGRAY);
            },
            TerrainType::Table => {
                self.draw_tile(ctx, TABLE, offset, &DARKGOLDENROD);
            },
            TerrainType::Fountain => {
                self.draw_tile(ctx, FOUNTAIN, offset, &GAINSBORO);
            },
            TerrainType::Altar => {
                self.draw_tile(ctx, ALTAR, offset, &GAINSBORO);
            },
            TerrainType::Barrel => {
                self.draw_tile(ctx, BARREL, offset, &DARKGOLDENROD);
            },
            TerrainType::Grave => {
                self.draw_tile(ctx, GRAVE, offset, &SLATEGRAY);
            },
            TerrainType::Stone => {
                self.draw_tile(ctx, STONE, offset, &SLATEGRAY);
            },
            TerrainType::Menhir => {
                self.draw_tile(ctx, MENHIR, offset, &SLATEGRAY);
            },
            TerrainType::DeadTree => {
                self.draw_tile(ctx, TREE_TRUNK, offset, &SADDLEBROWN);
            },
            TerrainType::TallGrass => {
                self.draw_tile(ctx, TALLGRASS, offset, &GOLD);
            },
        }


    }
    fn draw_block(&'a self, ctx: &mut Canvas, k: &Kernel<TerrainType>, offset: V2<f32>, idx: usize, color: &Rgb) {
        self.draw_tile(ctx, BLANK_FLOOR, offset, &BLACK);
        // Back lines for blocks with open floor behind them.
        if !k.nw.is_block() {
            self.draw_tile(ctx, BLOCK_NW, offset, color);
        }
        if !k.n.is_block() {
            self.draw_tile(ctx, BLOCK_N, offset, color);
        }
        if !k.ne.is_block() {
            self.draw_tile(ctx, BLOCK_NE, offset, color);
        }

        // Front faces if visible.
        if !k.sw.is_block() {
            self.draw_tile(ctx, idx, offset, color);
        }
        if !k.s.is_block() {
            self.draw_tile(ctx, idx + 1, offset, color);
        }
        if !k.se.is_block() {
            self.draw_tile(ctx, idx + 2, offset, color);
        }
    }

    fn draw_wall(&'a self, ctx: &mut Canvas, k: &Kernel<TerrainType>,
            offset: V2<f32>, idx: usize, color: &Rgb) {

        if !self.is_exposed_wall(self.loc) {
            self.draw_block(ctx, k, offset, BLANK_BLOCK, &BLACK);
            return;
        }

        // Create a connectivity bit mask where the bit positions match
        // the wall tile fragments indices.
        let flags =
            if self.is_exposed_wall(self.loc + V2(-1,  0)) { 1 } else { 0 } +
            if self.is_exposed_wall(self.loc + V2( 1,  0)) { 2 } else { 0 } +
            if self.is_exposed_wall(self.loc + V2( 0,  1)) { 4 } else { 0 } +
            if self.is_exposed_wall(self.loc + V2( 0, -1)) { 8 } else { 0 };

        // Draw the segments. Make sure the order does the back parts
        // before the front parts.
        for &i in [0, 3, 2, 1].iter() {
            if i == 2 {
                // Always draw the center pillar after the back walls are
                // drawn.
                self.draw_tile(ctx, idx + 4, offset, color);
                self.draw_tile(ctx, idx + 5, offset, color);
            }
            if flags & (1 << i) != 0 {
                self.draw_tile(ctx, idx + i, offset, color);
            }
        }
    }

    fn is_exposed_wall(&'a self, loc: Location) -> bool {
        loc.terrain().is_wall() && (
        !(loc + V2(-1, -1)).terrain().is_hull() ||
        !(loc + V2( 0, -1)).terrain().is_hull() ||
        !(loc + V2( 1, -1)).terrain().is_hull() ||
        !(loc + V2( 1,  0)).terrain().is_hull() ||
        !(loc + V2( 1,  1)).terrain().is_hull() ||
        !(loc + V2( 0,  1)).terrain().is_hull() ||
        !(loc + V2(-1,  1)).terrain().is_hull() ||
        !(loc + V2(-1,  0)).terrain().is_hull())
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
                self.draw_tile(ctx, icon, body_pos, &color);
                // Ground mound, doesn't bob.
                self.draw_tile(ctx, icon + 1, offset, &color);
            } else {
                self.draw_tile(ctx, icon, body_pos, &color);
            }
        }
    }
}

/// 3x3 grid of terrain cells. Use this as the input for terrain tile
/// computation, which will need to consider the immediate vicinity of cells.
struct Kernel<C> {
    n: C,
    ne: C,
    nw: C,
    center: C,
    se: C,
    sw: C,
    s: C,
}

impl<C: Clone> Kernel<C> {
    pub fn new<F>(get: F, loc: Location) -> Kernel<C>
        where F: Fn(Location) -> C {
        Kernel {
            n: get(loc + V2(-1, -1)),
            ne: get(loc + V2(0, -1)),
            nw: get(loc + V2(-1, 0)),
            center: get(loc),
            se: get(loc + V2(1, 0)),
            sw: get(loc + V2(0, 1)),
            s: get(loc + V2(1, 1)),
        }
    }
}
