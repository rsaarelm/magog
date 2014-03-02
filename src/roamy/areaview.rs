use cgmath::point::{Point, Point2};
use cgmath::vector::{Vec2};
use cgmath::aabb::{Aabb, Aabb2};
use color::rgb::RGB;
use color::rgb::consts::*;
use calx::rectutil::RectUtil;
use area;
use area::{TerrainType, Location, Area};
use fov::Fov;
use glutil::app::{App, SPRITE_INDEX_START};

pub static FLOOR : uint = SPRITE_INDEX_START + 10;
pub static CUBE : uint = SPRITE_INDEX_START + 1;
pub static XWALL : uint = SPRITE_INDEX_START + 20;
pub static YWALL : uint = SPRITE_INDEX_START + 21;
pub static XYWALL : uint = SPRITE_INDEX_START + 22;
pub static OWALL : uint = SPRITE_INDEX_START + 23;
pub static AVATAR : uint = SPRITE_INDEX_START + 26;
pub static WATER : uint = SPRITE_INDEX_START + 12;
pub static MAGMA : uint = SPRITE_INDEX_START + 13;
pub static MAGMA_2 : uint = SPRITE_INDEX_START + 7;
pub static CURSOR_BOTTOM : uint = SPRITE_INDEX_START + 8;
pub static CURSOR_TOP : uint = SPRITE_INDEX_START + 9;
pub static DOWNSTAIRS : uint = SPRITE_INDEX_START + 14;
pub static BLOCK : uint = SPRITE_INDEX_START + 27;
pub static TREE_TRUNK : uint = SPRITE_INDEX_START + 24;
pub static TREE_FOLIAGE : uint = SPRITE_INDEX_START + 32;
pub static GRASS : uint = SPRITE_INDEX_START + 11;

static WALL_COL: &'static RGB<u8> = &LIGHTSLATEGRAY;
static ROCK_COL: &'static RGB<u8> = &DARKGOLDENROD;
static CURSOR_COL: &'static RGB<u8> = &FIREBRICK;

static FLOOR_Z: f32 = 0.500f32;
static BLOCK_Z: f32 = 0.400f32;

/// 3x3 grid of terrain cells. Use this as the input for terrain sprite
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

impl<C> Kernel<C> {
    pub fn new(get: |Location| -> C, loc: Location) -> Kernel<C> {
        Kernel {
            n: get(loc + Vec2::new(-1, -1)),
            ne: get(loc + Vec2::new(0, -1)),
            e: get(loc + Vec2::new(1, -1)),
            nw: get(loc + Vec2::new(-1, 0)),
            center: get(loc),
            se: get(loc + Vec2::new(1, 0)),
            w: get(loc + Vec2::new(-1, 1)),
            sw: get(loc + Vec2::new(0, 1)),
            s: get(loc + Vec2::new(1, 1)),
        }
    }
}

pub struct Sprite {
    idx: uint,
    pos: Point2<f32>,
    z: f32,
    color: RGB<u8>,
}

impl Sprite {
    pub fn draw(&self, app: &mut App) {
        app.draw_sprite(self.idx, &self.pos, self.z, &self.color);
    }
}

pub fn terrain_sprites(k: &Kernel<TerrainType>, pos: &Point2<f32>) -> ~[Sprite] {
    let mut ret = ~[];

    // TODO: Make this thing more data-driven once the data schema needed by
    // different types of terrain becomes clearer.
    match k.center {
        area::Water => {
            ret.push(Sprite { idx: WATER, pos: *pos, z: FLOOR_Z, color: ROYALBLUE });
        },
        area::Magma => {
            // A two-tone tile, made using two sprites.
            ret.push(Sprite { idx: MAGMA, pos: *pos, z: FLOOR_Z, color: CRIMSON });
            ret.push(Sprite { idx: MAGMA_2, pos: *pos, z: FLOOR_Z, color: YELLOW });
        },
        area::Tree => {
            // A two-toner, with floor, using two z-layers
            ret.push(Sprite { idx: FLOOR, pos: *pos, z: FLOOR_Z, color: SLATEGRAY });
            ret.push(Sprite { idx: TREE_TRUNK, pos: *pos, z: BLOCK_Z, color: SADDLEBROWN });
            ret.push(Sprite { idx: TREE_FOLIAGE, pos: *pos, z: BLOCK_Z, color: DARKGREEN });
        },
        area::Floor => {
            ret.push(Sprite { idx: FLOOR, pos: *pos, z: FLOOR_Z, color: SLATEGRAY });
        },
        area::Grass => {
            ret.push(Sprite { idx: GRASS, pos: *pos, z: FLOOR_Z, color: DARKGREEN });
        },
        area::Downstairs => {
            ret.push(Sprite { idx: DOWNSTAIRS, pos: *pos, z: FLOOR_Z, color: SLATEGRAY });
        },
        area::Rock => {
            ret.push(Sprite { idx: BLOCK, pos: *pos, z: BLOCK_Z, color: *ROCK_COL });
            // Back lines for blocks with open floor behind them.
            if !k.nw.is_wall() {
                ret.push(Sprite { idx: BLOCK + 1, pos: *pos, z: BLOCK_Z, color: *ROCK_COL });
            }
            if !k.n.is_wall() {
                ret.push(Sprite { idx: BLOCK + 2, pos: *pos, z: BLOCK_Z, color: *ROCK_COL });
            }
            if !k.ne.is_wall() {
                ret.push(Sprite { idx: BLOCK + 3, pos: *pos, z: BLOCK_Z, color: *ROCK_COL });
            }
        }
        area::Wall => {
            ret.push(Sprite { idx: FLOOR, pos: *pos, z: FLOOR_Z, color: SLATEGRAY });
            let (left_wall, right_wall, block) = wall_flags_lrb(k);
            if block {
                ret.push(Sprite { idx: CUBE, pos: *pos, z: BLOCK_Z, color: *WALL_COL });
            }
            if left_wall && right_wall {
                ret.push(Sprite { idx: XYWALL, pos: *pos, z: BLOCK_Z, color: *WALL_COL });
            } else if left_wall {
                ret.push(Sprite { idx: XWALL, pos: *pos, z: BLOCK_Z, color: *WALL_COL });
            } else if right_wall {
                ret.push(Sprite { idx: YWALL, pos: *pos, z: BLOCK_Z, color: *WALL_COL });
            } else if !block || !k.s.is_wall() {
                // NB: This branch has some actual local kernel logic not
                // handled by wall_flags_lrb.
                ret.push(Sprite { idx: OWALL, pos: *pos, z: BLOCK_Z, color: *WALL_COL });
            }
        },
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

    ret
}

pub fn draw_area(
    area: &Area, app: &mut App, center: &Location,
    seen: &Fov, remembered: &Fov) {
    // XXX: Horrible prototype code, figure out cleaning.

    let origin = Vec2::new(320.0f32, 180.0f32);

    // Mouse cursoring
    let mouse = app.get_mouse();
    let cursor_chart_pos = screen_to_chart(&mouse.pos.add_v(&origin.neg()).add_v(&Vec2::new(8.0f32, 0.0f32)));

    let mut rect = Aabb2::new(
        screen_to_chart(&Point2::new(0f32, 0f32).add_v(&origin.neg())),
        screen_to_chart(&Point2::new(640f32, 392f32).add_v(&origin.neg())));
    rect = rect.grow(&screen_to_chart(&Point2::new(640f32, 0f32).add_v(&origin.neg())));
    rect = rect.grow(&screen_to_chart(&Point2::new(0f32, 392f32).add_v(&origin.neg())));

    let &Location(ref offset) = center;
    let pos_offset = Vec2::new(offset.x as int, offset.y as int);

    for pt in rect.points() {
        let p = Location(pt) + pos_offset;
        let offset = chart_to_screen(&pt).add_v(&origin);

        let seen = seen.contains(&p) || remembered.contains(&p);
        let kernel = Kernel::new(|p| area.get(p), p);
        let mut sprites = terrain_sprites(&kernel, &offset);
        for s in sprites.mut_iter() {
            if !seen { s.color = DARKSLATEGRAY; }
            s.draw(app);
        }

        if &p == center {
            app.draw_sprite(AVATAR, &offset, BLOCK_Z, &AZURE);
        }
    }

    app.draw_sprite(CURSOR_BOTTOM, &chart_to_screen(&cursor_chart_pos).add_v(&origin), FLOOR_Z, CURSOR_COL);
    app.draw_sprite(CURSOR_TOP, &chart_to_screen(&cursor_chart_pos).add_v(&origin), BLOCK_Z, CURSOR_COL);

}

pub fn chart_to_screen(map_pos: &Point2<i8>) -> Point2<f32> {
    Point2::new(
        16.0 * (map_pos.x as f32) - 16.0 * (map_pos.y as f32),
        8.0 * (map_pos.x as f32) + 8.0 * (map_pos.y as f32))
}

pub fn screen_to_chart(screen_pos: &Point2<f32>) -> Point2<i8> {
    let column = (screen_pos.x / 16.0).floor();
    let row = ((screen_pos.y - column * 8.0) / 16.0).floor();
    Point2::new((column + row) as i8, row as i8)
}
