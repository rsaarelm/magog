use cgmath::point::{Point2};
use cgmath::vector::{Vec2};
use cgmath::aabb::{Aabb, Aabb2};
use color::rgb::RGB;
use color::rgb::consts::*;
use calx::rectutil::RectUtil;
use calx::app::App;
use calx::app::{SPRITE_INDEX_START};
use calx::renderer::Renderer;
use area;
use area::{TerrainType, Location};
use game::Game;
use transform::Transform;
use sprite::{Sprite, BLOCK_Z, FLOOR_Z};

pub static CUBE : uint = SPRITE_INDEX_START + 0;
pub static CURSOR_BOTTOM : uint = SPRITE_INDEX_START + 1;
pub static CURSOR_TOP : uint = SPRITE_INDEX_START + 2;
pub static BLOCK_NW : uint = SPRITE_INDEX_START + 3;
pub static BLOCK_N : uint = SPRITE_INDEX_START + 4;
pub static BLOCK_NE : uint = SPRITE_INDEX_START + 5;
pub static BLOCK_DARK : uint = SPRITE_INDEX_START + 6;
pub static BLANK_FLOOR : uint = SPRITE_INDEX_START + 10;
pub static FLOOR : uint = SPRITE_INDEX_START + 11;
pub static GRASS : uint = SPRITE_INDEX_START + 12;
pub static WATER : uint = SPRITE_INDEX_START + 13;
pub static MAGMA : uint = SPRITE_INDEX_START + 14;
pub static DOWNSTAIRS : uint = SPRITE_INDEX_START + 15;
pub static ROCKWALL : uint = SPRITE_INDEX_START + 16;
pub static WALL : uint = SPRITE_INDEX_START + 20;
pub static TREE_TRUNK : uint = SPRITE_INDEX_START + 48;
pub static TREE_FOLIAGE : uint = SPRITE_INDEX_START + 49;
pub static AVATAR : uint = SPRITE_INDEX_START + 51;
pub static BLOCK : uint = SPRITE_INDEX_START + 52;
pub static STALAGMITE : uint = SPRITE_INDEX_START + 56;

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

pub fn terrain_sprites(k: &Kernel<TerrainType>, pos: &Point2<f32>) -> ~[Sprite] {
    let mut ret = ~[];

    // TODO: Make this thing more data-driven once the data schema needed by
    // different types of terrain becomes clearer.
    match k.center {
        area::Water => {
            ret.push(Sprite { idx: WATER, pos: *pos, z: FLOOR_Z, color: ROYALBLUE });
        },
        area::Magma => {
            ret.push(Sprite { idx: MAGMA, pos: *pos, z: FLOOR_Z, color: DARKRED });
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
            ret.push(Sprite { idx: FLOOR, pos: *pos, z: FLOOR_Z, color: SLATEGRAY });
            ret.push(Sprite { idx: DOWNSTAIRS, pos: *pos, z: BLOCK_Z, color: SLATEGRAY });
        },
        area::Rock => {
            blockform(k, &mut ret, pos, BLOCK, &DARKGOLDENROD);
        }
        area::Wall => {
            ret.push(Sprite { idx: FLOOR, pos: *pos, z: FLOOR_Z, color: SLATEGRAY });
            wallform(k, &mut ret, pos, WALL, &LIGHTSLATEGRAY);
        },
        area::RockWall => {
            ret.push(Sprite { idx: FLOOR, pos: *pos, z: FLOOR_Z, color: SLATEGRAY });
            wallform(k, &mut ret, pos, ROCKWALL, &LIGHTSLATEGRAY);
        },
        area::Stalagmite => {
            ret.push(Sprite { idx: FLOOR, pos: *pos, z: FLOOR_Z, color: SLATEGRAY });
            ret.push(Sprite { idx: STALAGMITE, pos: *pos, z: BLOCK_Z, color: DARKGOLDENROD });
        },
    }

    fn blockform(k: &Kernel<TerrainType>, ret: &mut ~[Sprite], pos: &Point2<f32>, idx: uint, color: &RGB<u8>) {
        ret.push(Sprite { idx: idx, pos: *pos, z: BLOCK_Z, color: *color });
        // Back lines for blocks with open floor behind them.
        if !k.nw.is_wall() {
            ret.push(Sprite { idx: BLOCK_NW, pos: *pos, z: BLOCK_Z, color: *color });
        }
        if !k.n.is_wall() {
            ret.push(Sprite { idx: BLOCK_N, pos: *pos, z: BLOCK_Z, color: *color });
        }
        if !k.ne.is_wall() {
            ret.push(Sprite { idx: BLOCK_NE, pos: *pos, z: BLOCK_Z, color: *color });
        }
    }

    fn wallform(k: &Kernel<TerrainType>, ret: &mut ~[Sprite], pos: &Point2<f32>, idx: uint, color: &RGB<u8>) {
        let (left_wall, right_wall, block) = wall_flags_lrb(k);
        if block {
            // TODO: See-through walls should be drawn differently, don't show the blocked
            // innards, just an expanse of XYWALL.
            //   The logic's in place below, but this doesn't make sense until areaview
            // is expanded to handle multiple wall types.
            if area::Wall.is_opaque() {
                ret.push(Sprite { idx: CUBE, pos: *pos, z: BLOCK_Z, color: *color });
            } else {
                ret.push(Sprite { idx: idx + 2, pos: *pos, z: BLOCK_Z, color: *color });
                return;
            }
        }
        if left_wall && right_wall {
            ret.push(Sprite { idx: idx + 2, pos: *pos, z: BLOCK_Z, color: *color });
        } else if left_wall {
            ret.push(Sprite { idx: idx, pos: *pos, z: BLOCK_Z, color: *color });
        } else if right_wall {
            ret.push(Sprite { idx: idx + 1, pos: *pos, z: BLOCK_Z, color: *color });
        } else if !block || !k.s.is_wall() {
            // NB: This branch has some actual local kernel logic not
            // handled by wall_flags_lrb.
            ret.push(Sprite { idx: idx + 3, pos: *pos, z: BLOCK_Z, color: *color });
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

    ret
}

pub fn draw_area<R: Renderer>(game: &mut Game, app: &mut App<R>) {

    let xf = Transform::new(game.pos);

    let mut rect = Aabb2::new(
        *xf.to_chart(&Point2::new(0f32, 0f32)).p(),
        *xf.to_chart(&Point2::new(640f32, 392f32)).p());
    rect = rect.grow(xf.to_chart(&Point2::new(640f32, 0f32)).p());
    rect = rect.grow(xf.to_chart(&Point2::new(0f32, 392f32)).p());

    for pt in rect.points() {
        let p = Location(pt);
        let offset = xf.to_screen(p);

        let kernel = Kernel::new(|p| game.area.get(p), p);
        let mut sprites = terrain_sprites(&kernel, &offset);
        if !game.seen.contains(p) {
            if game.remembered.contains(p) {
                for s in sprites.mut_iter() {
                    s.color = RGB::new(0x22u8, 0x22u8, 0x11u8);
                }
            } else {
                // Solid blocks for unseen areas, cover stuff in front.
                sprites = ~[Sprite { idx: BLOCK_DARK, pos: offset, z: BLOCK_Z, color: BLACK }];
            }
        }

        for s in sprites.iter() {
            s.draw(app);
        }

        if game.seen.contains(p) {
            match game.mob_at(p) {
                Some(mob) => {
                    for s in mob.sprites(&xf).iter() {
                        s.draw(app);
                    }
                }
                _ => ()
            };
        }
    }
}
