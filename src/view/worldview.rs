use calx::color::*;
use calx::{Context, V2};
use calx::timing;
use cgmath::{Aabb, Aabb2, Vector, Vector2};
use time;
use view::tilecache;
use view::tilecache::tile::*;
use view::drawable::{Drawable, Translated};
use world::area::Area;
use world::fov::{Fov, FovStatus, Seen, Remembered};
use world::mobs::{Mobs, Mob, MobType};
use world::mobs;
use world::terrain::TerrainType;
use world::terrain;
use world::spatial::{Location};
use world::system::{World, Entity};

pub static FLOOR_Z: f32 = 0.500f32;
pub static BLOCK_Z: f32 = 0.400f32;
pub static FX_Z: f32 = 0.375f32;
pub static CAPTION_Z: f32 = 0.350f32;

// TODO: Replace with calx::V2.
fn v2<T>(x: T, y: T) -> Vector2<T> { Vector2::new(x, y) }

/// Drawable representation of a single map location.
pub struct CellDrawable {
    loc: Location,
    fov: Option<FovStatus>,
    world: World,
}

impl Drawable for CellDrawable {
    fn draw(&self, ctx: &mut Context, offset: &V2<int>) {
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
            let nw = c.loc + v2(-1, 0);
            let ne = c.loc + v2(0, -1);

            for &p in vec![nw, ne].iter() {
                let t = c.world.terrain_at(p);
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
    pub fn new(loc: Location, fov: Option<FovStatus>, world: World) -> CellDrawable {
        CellDrawable {
            loc: loc,
            fov: fov,
            world: world,
        }
    }

    fn draw_tile(&self, ctx: &mut Context, idx: uint, offset: &V2<int>, z: f32, color: &Rgb) {
        let color = match self.fov {
            Some(Remembered) => Rgb::new(0x22u8, 0x22u8, 0x11u8),
            _ => *color,
        };

        // TODO
        /*
        ctx.set_layer(z);
        ctx.set_color(&color);
        ctx.draw_image(&tilecache::get(idx), offset);
        */
    }

    fn draw_cell(&self, ctx: &mut Context, offset: &V2<int>) {
        self.draw_terrain(ctx, offset);

        if self.fov == Some(Seen) {
            for mob in self.world.mobs_at(self.loc).iter() {
                self.draw_mob(ctx, offset, mob);
            }
        }
    }

    fn draw_terrain(&self, ctx: &mut Context, offset: &V2<int>) {
        let k = Kernel::new(|loc| self.world.terrain_at(loc), self.loc);
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

        fn blockform(c: &CellDrawable, ctx: &mut Context, k: &Kernel<TerrainType>, offset: &V2<int>, idx: uint, color: &Rgb) {
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

        fn wallform(c: &CellDrawable, ctx: &mut Context, k: &Kernel<TerrainType>, offset: &V2<int>, idx: uint, color: &Rgb, opaque: bool) {
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

    fn draw_mob(&self, ctx: &mut Context, offset: &V2<int>, mob: &Entity) {
        let body_pos =
            if is_bobbing(mob) {
                offset.add_v(timing::cycle_anim(
                        0.3f64,
                        &[v2(0.0f32, 0.0f32), v2(0.0f32, -1.0f32)]))
            } else { *offset };

        let (icon, color) = visual(mob.mob_type());
        match mob.mob_type() {
            mobs::Serpent => {
                // Body
                self.draw_tile(ctx, 94, &body_pos, BLOCK_Z, &color);
                // Ground mound
                self.draw_tile(ctx, 95, offset, BLOCK_Z, &color);
            }
            _ => {
                self.draw_tile(ctx, icon, &body_pos, BLOCK_Z, &color);
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
            n: get(loc + v2(-1, -1)),
            ne: get(loc + v2(0, -1)),
            e: get(loc + v2(1, -1)),
            nw: get(loc + v2(-1, 0)),
            center: get(loc),
            se: get(loc + v2(1, 0)),
            w: get(loc + v2(-1, 1)),
            sw: get(loc + v2(0, 1)),
            s: get(loc + v2(1, 1)),
        }
    }
}

pub fn draw_area(
    world: &World, ctx: &mut Context, center: Location, fov: &Fov) {
    let mut bounds = Aabb2::new(
        screen_to_loc(center, &v2(0f32, 0f32)).to_point(),
        screen_to_loc(center, &v2(640f32, 392f32)).to_point());
    bounds = bounds.grow(
        &screen_to_loc(center, &v2(640f32, 0f32)).to_point());
    bounds = bounds.grow(
        &screen_to_loc(center, &v2(0f32, 392f32)).to_point());

    for pt in bounds.points() {
        let loc = Location::new(pt.x as i8, pt.y as i8);

        let drawable = Translated::new(loc_to_view(loc),
        CellDrawable::new(loc, fov.get(loc), world.clone()));
        drawable.draw(ctx, &view_to_screen(&loc_to_view(center), &v2(0f32, 0f32)));
    }
}


pub fn draw_mouse(ctx: &mut Context, center_loc: Location) -> Location {
    let mouse = ctx.get_mouse();
    let cursor_loc = screen_to_loc(center_loc, &mouse.pos);
    let draw_pos = loc_to_screen(center_loc, cursor_loc);

    // TODO
    /*
    ctx.set_color(&FIREBRICK);
    ctx.set_layer(FLOOR_Z);
    ctx.draw_image(&tilecache::get(CURSOR_BOTTOM), &draw_pos);
    ctx.set_layer(BLOCK_Z);
    ctx.draw_image(&tilecache::get(CURSOR_TOP), &draw_pos);
    */

    cursor_loc
}

static CENTER_X: f32 = 320.0;
static CENTER_Y: f32 = 180.0;

/// Convert a location into absolute view space coordinates.
pub fn loc_to_view(loc: Location) -> V2<int> {
    let x = (loc.x) as f32;
    let y = (loc.y) as f32;
    v2(16.0 * x - 16.0 * y, 8.0 * x + 8.0 * y)
}

/// Convert absolute view space coordinates into a Location.
pub fn view_to_loc(view_pos: &V2<int>) -> Location {
    let column = ((view_pos.x + 8.0) / 16.0).floor();
    let row = ((view_pos.y as f32 - column * 8.0) / 16.0).floor();
    Location::new((column + row) as i8, row as i8)
}

/// Convert absolute view space coordinates into screen coordinates centered around a given view
/// position.
pub fn view_to_screen(view_center: &V2<int>, view_pos: &V2<int>) -> V2<int> {
    view_pos.sub_v(view_center).add_v(&v2(CENTER_X, CENTER_Y))
}

/// Convert screen coordinates centered around a given view position to absolute view coordinates.
pub fn screen_to_view(view_center: &V2<int>, screen_pos: &V2<int>) -> V2<int> {
    screen_pos.add_v(view_center).sub_v(&v2(CENTER_X, CENTER_Y))
}

/// Convert screen coordinates to location.
pub fn screen_to_loc(center_loc: Location, screen_pos: &V2<int>) -> Location {
    view_to_loc(&screen_to_view(&loc_to_view(center_loc), screen_pos))
}

/// Convert location to screen coordinates.
pub fn loc_to_screen(center_loc: Location, loc: Location) -> V2<int> {
    view_to_screen(&loc_to_view(center_loc), &loc_to_view(loc))
}
