use calx::color::consts::*;
use calx::color::{RGB};
use calx::engine::{Engine};
use calx::rectutil::RectUtil;
use calx::timing;
use cgmath::aabb::{Aabb, Aabb2};
use cgmath::vector::{Vector, Vector2};
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
use world::spatial::{Location, ChartPos};
use world::system::{World, Entity};

pub static FLOOR_Z: f32 = 0.500f32;
pub static BLOCK_Z: f32 = 0.400f32;
pub static FX_Z: f32 = 0.375f32;
pub static FOG_Z: f32 = 0.350f32;
pub static CAPTION_Z: f32 = 0.300f32;

pub struct CellDrawable {
    loc: Location,
    fov: Option<FovStatus>,
    world: World,
}

impl Drawable for CellDrawable {
    fn draw(&self, ctx: &mut Engine, offset: &Vector2<f32>) {
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
            let nw = c.loc + Vector2::new(-1, 0);
            let ne = c.loc + Vector2::new(0, -1);

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

    fn draw_tile(&self, ctx: &mut Engine, idx: uint, offset: &Vector2<f32>, z: f32, color: &RGB) {
        let color = match self.fov {
            Some(Remembered) => RGB::new(0x22u8, 0x22u8, 0x11u8),
            _ => *color,
        };

        ctx.set_layer(z);
        ctx.set_color(&color);
        ctx.draw_image(&tilecache::get(idx), offset);
    }

    fn draw_cell(&self, ctx: &mut Engine, offset: &Vector2<f32>) {
        self.draw_terrain(ctx, offset);

        if self.fov == Some(Seen) {
            for mob in self.world.mobs_at(self.loc).iter() {
                self.draw_mob(ctx, offset, mob);
            }
        }
    }

    fn draw_terrain(&self, ctx: &mut Engine, offset: &Vector2<f32>) {
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
                let portal_col = RGB::new(glow, glow, 255);
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

        fn blockform(c: &CellDrawable, ctx: &mut Engine, k: &Kernel<TerrainType>, offset: &Vector2<f32>, idx: uint, color: &RGB) {
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

        fn wallform(c: &CellDrawable, ctx: &mut Engine, k: &Kernel<TerrainType>, offset: &Vector2<f32>, idx: uint, color: &RGB, opaque: bool) {
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

    fn draw_mob(&self, ctx: &mut Engine, offset: &Vector2<f32>, mob: &Entity) {
        let body_pos =
            if is_bobbing(mob) {
                offset.add_v(timing::cycle_anim(
                        0.3f64,
                        &[Vector2::new(0.0f32, 0.0f32), Vector2::new(0.0f32, -1.0f32)]))
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

        fn visual(t: MobType) -> (uint, RGB) {
            match t {
                mobs::Player => (51, AZURE),
                mobs::Dreg => (72, OLIVE),
                mobs::GridBug => (76, MAGENTA),
                mobs::Serpent => (94, CORAL),
            }
        }

        fn is_bobbing(mob: &Entity) -> bool {
            // TODO: Sleeping mobs don't bob.
            mob.mob_type() != mobs::Player
        }
    }
}

/// 3x3 grid of terrain cells. Use this as the input for terrain tile
/// computation, which will need to consider the immediate vicinity of cells.
pub struct Kernel<C> {
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
            n: get(loc + Vector2::new(-1, -1)),
            ne: get(loc + Vector2::new(0, -1)),
            e: get(loc + Vector2::new(1, -1)),
            nw: get(loc + Vector2::new(-1, 0)),
            center: get(loc),
            se: get(loc + Vector2::new(1, 0)),
            w: get(loc + Vector2::new(-1, 1)),
            sw: get(loc + Vector2::new(0, 1)),
            s: get(loc + Vector2::new(1, 1)),
        }
    }

    pub fn new_default(center: C, edge: C) -> Kernel<C> {
        Kernel {
            n: edge.clone(),
            ne: edge.clone(),
            e: edge.clone(),
            nw: edge.clone(),
            center: center,
            se: edge.clone(),
            w: edge.clone(),
            sw: edge.clone(),
            s: edge.clone(),
        }
    }
}

pub trait WorldView {
    fn draw_area(
        &self, ctx: &mut Engine, center: Location, fov: &Fov);
}

impl WorldView for World {
    fn draw_area(
        &self, ctx: &mut Engine, center: Location, fov: &Fov) {
        let mut chart_bounds = Aabb2::new(
            to_chart(&Vector2::new(0f32, 0f32)).to_point(),
            to_chart(&Vector2::new(640f32, 392f32)).to_point());
        chart_bounds = chart_bounds.grow(&to_chart(&Vector2::new(640f32, 0f32)).to_point());
        chart_bounds = chart_bounds.grow(&to_chart(&Vector2::new(0f32, 392f32)).to_point());

        for pt in chart_bounds.points() {
            let p = ChartPos::new(pt.x, pt.y);
            let offset = to_screen(p);
            let loc = Location::new(center.x + p.x as i8, center.y + p.y as i8);

            // TODO: Offset for Translated into fixed translation view space,
            // center to screen with the later draw call offset.
            let drawable = Translated::new(
                offset, CellDrawable::new(loc, fov.get(loc), self.clone()));
            drawable.draw(ctx, &Vector2::new(0f32, 0f32));
        }
    }
}


pub fn draw_mouse(ctx: &mut Engine) -> ChartPos {
    let mouse = ctx.get_mouse();
    let cursor_chart_pos = to_chart(&mouse.pos);

    ctx.set_color(&FIREBRICK);
    ctx.set_layer(FLOOR_Z);
    ctx.draw_image(&tilecache::get(CURSOR_BOTTOM), &to_screen(cursor_chart_pos));
    ctx.set_layer(BLOCK_Z);
    ctx.draw_image(&tilecache::get(CURSOR_TOP), &to_screen(cursor_chart_pos));

    cursor_chart_pos
}

static CENTER_X: f32 = 320.0;
static CENTER_Y: f32 = 180.0;

fn to_screen(pos: ChartPos) -> Vector2<f32> {
    let x = (pos.x) as f32;
    let y = (pos.y) as f32;
    Vector2::new(CENTER_X + 16.0 * x - 16.0 * y, CENTER_Y + 8.0 * x + 8.0 * y)
}

fn to_chart(pos: &Vector2<f32>) -> ChartPos {
    let column = ((pos.x + 8.0 - CENTER_X) / 16.0).floor();
    let row = ((pos.y - CENTER_Y as f32 - column * 8.0) / 16.0).floor();
    ChartPos::new((column + row) as int, row as int)
}
