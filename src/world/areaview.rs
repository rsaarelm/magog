use time;
use cgmath::point::{Point2};
use cgmath::vector::{Vector2};
use cgmath::aabb::{Aabb, Aabb2};
use color::rgb::{RGB, ToRGB};
use color::rgb::consts::*;
use stb::image;
use rectutil::RectUtil;
use engine::{App, Engine, Image};
use tile::Tile;
use world::area;
use world::area::{TerrainType, Location, ChartPos};
use world::fov;
use world::sprite::{Sprite, BLOCK_Z, FLOOR_Z, DrawContext};
use world::state::State;
use world::transform::Transform;

static TILE_DATA: &'static [u8] = include_bin!("../../assets/tile.png");

pub static CUBE : uint = 0;
pub static CURSOR_BOTTOM : uint = 1;
pub static CURSOR_TOP : uint = 2;
pub static BLOCK_NW : uint = 3;
pub static BLOCK_N : uint = 4;
pub static BLOCK_NE : uint = 5;
pub static BLOCK_DARK : uint = 6;
pub static CHASM : uint = 7;
pub static SHALLOWS : uint = 8;
pub static PORTAL : uint = 9;
pub static BLANK_FLOOR : uint = 10;
pub static FLOOR : uint = 11;
pub static GRASS : uint = 12;
pub static WATER : uint = 13;
pub static MAGMA : uint = 14;
pub static DOWNSTAIRS : uint = 15;
pub static ROCKWALL : uint = 16;
pub static WALL : uint = 20;
pub static FENCE : uint = 24;
pub static BARS : uint = 28;
pub static WINDOW : uint = 32;
pub static DOOR : uint = 36;
pub static TREE_TRUNK : uint = 48;
pub static TREE_FOLIAGE : uint = 49;
pub static TABLE : uint = 50;
pub static AVATAR : uint = 51;
pub static BLOCK : uint = 52;
pub static FOUNTAIN : uint = 53;
pub static ALTAR : uint = 54;
pub static BARREL : uint = 55;
pub static STALAGMITE : uint = 56;
pub static GRAVE : uint = 58;
pub static STONE : uint = 69;
pub static MENHIR : uint = 70;
pub static TALLGRASS : uint = 80;

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

pub fn terrain_sprites<C: DrawContext>(
    ctx: &mut C, k: &Kernel<TerrainType>, pos: &Point2<f32>) {
    match k.center {
        area::Void => {
            ctx.draw(BLANK_FLOOR, pos, FLOOR_Z, &BLACK);
        },
        area::Water => {
            ctx.draw(WATER, pos, FLOOR_Z, &ROYALBLUE);
        },
        area::Shallows => {
            ctx.draw(SHALLOWS, pos, FLOOR_Z, &CORNFLOWERBLUE);
        },
        area::Magma => {
            ctx.draw(MAGMA, pos, FLOOR_Z, &DARKRED);
        },
        area::Tree => {
            // A two-toner, with floor, using two z-layers
            ctx.draw(FLOOR, pos, FLOOR_Z, &SLATEGRAY);
            ctx.draw(TREE_TRUNK, pos, BLOCK_Z, &SADDLEBROWN);
            ctx.draw(TREE_FOLIAGE, pos, BLOCK_Z, &GREEN);
        },
        area::Floor => {
            ctx.draw(FLOOR, pos, FLOOR_Z, &SLATEGRAY);
        },
        area::Chasm => {
            ctx.draw(CHASM, pos, FLOOR_Z, &DARKSLATEGRAY);
        },
        area::Grass => {
            ctx.draw(GRASS, pos, FLOOR_Z, &DARKGREEN);
        },
        area::Downstairs => {
            ctx.draw(FLOOR, pos, FLOOR_Z, &SLATEGRAY);
            ctx.draw(DOWNSTAIRS, pos, BLOCK_Z, &SLATEGRAY);
        },
        area::Portal => {
            let glow = (127.0 *(1.0 + (time::precise_time_s()).sin())) as u8;
            let portal_col = RGB::new(glow, glow, 255);
            ctx.draw(PORTAL, pos, BLOCK_Z, &portal_col);
        },
        area::Rock => {
            blockform(ctx, k, pos, BLOCK, &DARKGOLDENROD);
        }
        area::Wall => {
            ctx.draw(FLOOR, pos, FLOOR_Z, &SLATEGRAY);
            wallform(ctx, k, pos, WALL, &LIGHTSLATEGRAY, true);
        },
        area::RockWall => {
            ctx.draw(FLOOR, pos, FLOOR_Z, &SLATEGRAY);
            wallform(ctx, k, pos, ROCKWALL, &LIGHTSLATEGRAY, true);
        },
        area::Fence => {
            // The floor type beneath the fence tile is visible, make it grass if there's grass
            // behind the fence. Otherwise make it regular floor.
            if k.n == area::Grass || k.ne == area::Grass || k.nw == area::Grass {
                ctx.draw(GRASS, pos, FLOOR_Z, &DARKGREEN);
            } else {
                ctx.draw(FLOOR, pos, FLOOR_Z, &SLATEGRAY);
            }
            wallform(ctx, k, pos, FENCE, &DARKGOLDENROD, false);
        },
        area::Bars => {
            ctx.draw(FLOOR, pos, FLOOR_Z, &SLATEGRAY);
            wallform(ctx, k, pos, BARS, &GAINSBORO, false);
        },
        area::Stalagmite => {
            ctx.draw(FLOOR, pos, FLOOR_Z, &SLATEGRAY);
            ctx.draw(STALAGMITE, pos, BLOCK_Z, &DARKGOLDENROD);
        },
        area::Window => {
            ctx.draw(FLOOR, pos, FLOOR_Z, &SLATEGRAY);
            wallform(ctx, k, pos, WINDOW, &LIGHTSLATEGRAY, false);
        },
        area::Door => {
            ctx.draw(FLOOR, pos, FLOOR_Z, &SLATEGRAY);
            wallform(ctx, k, pos, DOOR, &LIGHTSLATEGRAY, true);
            wallform(ctx, k, pos, DOOR + 4, &SADDLEBROWN, false);
        },
        area::Table => {
            ctx.draw(FLOOR, pos, FLOOR_Z, &SLATEGRAY);
            ctx.draw(TABLE, pos, BLOCK_Z, &DARKGOLDENROD);
        },
        area::Fountain => {
            ctx.draw(FLOOR, pos, FLOOR_Z, &SLATEGRAY);
            ctx.draw(FOUNTAIN, pos, BLOCK_Z, &GAINSBORO);
        },
        area::Altar => {
            ctx.draw(FLOOR, pos, FLOOR_Z, &SLATEGRAY);
            ctx.draw(ALTAR, pos, BLOCK_Z, &GAINSBORO);
        },
        area::Barrel => {
            ctx.draw(FLOOR, pos, FLOOR_Z, &SLATEGRAY);
            ctx.draw(BARREL, pos, BLOCK_Z, &DARKGOLDENROD);
        },
        area::Grave => {
            ctx.draw(FLOOR, pos, FLOOR_Z, &SLATEGRAY);
            ctx.draw(GRAVE, pos, BLOCK_Z, &SLATEGRAY);
        },
        area::Stone => {
            ctx.draw(FLOOR, pos, FLOOR_Z, &SLATEGRAY);
            ctx.draw(STONE, pos, BLOCK_Z, &SLATEGRAY);
        },
        area::Menhir => {
            ctx.draw(FLOOR, pos, FLOOR_Z, &SLATEGRAY);
            ctx.draw(MENHIR, pos, BLOCK_Z, &SLATEGRAY);
        },
        area::DeadTree => {
            ctx.draw(FLOOR, pos, FLOOR_Z, &SLATEGRAY);
            ctx.draw(TREE_TRUNK, pos, BLOCK_Z, &SADDLEBROWN);
        },
        area::TallGrass => {
            ctx.draw(TALLGRASS, pos, BLOCK_Z, &GOLD);
        },
    }

    fn blockform<C: DrawContext>(ctx: &mut C, k: &Kernel<TerrainType>, pos: &Point2<f32>, idx: uint, color: &RGB<u8>) {
        ctx.draw(idx, pos, BLOCK_Z, color);
        // Back lines for blocks with open floor behind them.
        if !k.nw.is_wall() {
            ctx.draw(BLOCK_NW, pos, BLOCK_Z, color);
        }
        if !k.n.is_wall() {
            ctx.draw(BLOCK_N, pos, BLOCK_Z, color);
        }
        if !k.ne.is_wall() {
            ctx.draw(BLOCK_NE, pos, BLOCK_Z, color);
        }
    }

    fn wallform<C: DrawContext>(ctx: &mut C, k: &Kernel<TerrainType>, pos: &Point2<f32>, idx: uint, color: &RGB<u8>, opaque: bool) {
        let (left_wall, right_wall, block) = wall_flags_lrb(k);
        if block {
            if opaque {
                ctx.draw(CUBE, pos, BLOCK_Z, color);
            } else {
                ctx.draw(idx + 2, pos, BLOCK_Z, color);
                return;
            }
        }
        if left_wall && right_wall {
            ctx.draw(idx + 2, pos, BLOCK_Z, color);
        } else if left_wall {
            ctx.draw(idx, pos, BLOCK_Z, color);
        } else if right_wall {
            ctx.draw(idx + 1, pos, BLOCK_Z, color);
        } else if !block || !k.s.is_wall() {
            // NB: This branch has some actual local kernel logic not
            // handled by wall_flags_lrb.
            ctx.draw(idx + 3, pos, BLOCK_Z, color);
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

// TODO: Set up invariants so that draw_area cannot be called unless the tile
// set is set up.
pub fn init_tiles(ctx: &mut Engine) -> Vec<Image> {
    let tiles = image::Image::load_from_memory(TILE_DATA, 1).unwrap();
    let tiles = Tile::new_alpha_set(
        &Vector2::new(32, 32),
        &Vector2::new(tiles.width as int, tiles.height as int),
        tiles.pixels,
        &Vector2::new(-16, -16));
    ctx.make_images(&tiles)
}

pub struct SpriteCollector<'a> {
    pub sprites: Vec<Sprite>,
    pub mode: SpriteMode,
    engine: &'a mut Engine,
    tiles: &'a Vec<Image>,
}

pub enum SpriteMode {
    Normal,
    FogOfWar,
}

impl<'a> SpriteCollector<'a> {
    pub fn new<'a>(engine: &'a mut Engine, tiles: &'a Vec<Image>) -> SpriteCollector<'a> {
        SpriteCollector {
            sprites: vec!(),
            mode: Normal,
            engine: engine,
            tiles: tiles,
        }
    }
}

impl<'a> DrawContext for SpriteCollector<'a> {
    fn draw<C: ToRGB>(
        &mut self, idx: uint, pos: &Point2<f32>, z: f32, color: &C) {
        let color = match self.mode {
            Normal => color.to_rgb::<u8>(),
            FogOfWar => RGB::new(0x22u8, 0x22u8, 0x11u8),
        };

        self.engine.set_layer(z);
        self.engine.set_color(&color);
        self.engine.draw_image(self.tiles.get(idx), pos);
    }
}

pub fn draw_area<S: State>(ctx: &mut Engine, tiles: &Vec<Image>, state: &S) {
    let xf = state.transform();

    let mut rect = Aabb2::new(
        *xf.to_chart(&Point2::new(0f32, 0f32)).p(),
        *xf.to_chart(&Point2::new(640f32, 392f32)).p());
    rect = rect.grow(xf.to_chart(&Point2::new(640f32, 0f32)).p());
    rect = rect.grow(xf.to_chart(&Point2::new(0f32, 392f32)).p());

    for pt in rect.points() {
        let p = ChartPos::new(pt.x, pt.y);
        let offset = xf.to_screen(p);

        let loc = p.to_location();
        let kernel = Kernel::new(|p| state.area().get(p), loc);
        let mut acc = SpriteCollector::new(ctx, tiles);

        let fov = state.fov(loc);

        if fov == fov::Unknown {
            // Solid blocks for unseen areas, cover stuff in front.
            acc.draw(BLOCK_DARK, &offset, BLOCK_Z, &BLACK);
        } else {
            if fov == fov::Remembered { acc.mode = FogOfWar }
            terrain_sprites(&mut acc, &kernel, &offset);
        }

        if fov == fov::Seen {
            match state.drawable_mob_at(loc) {
                Some(mob) => mob.draw(&mut acc, &offset),
                _ => ()
            };
        }
    }
}

pub fn draw_mouse(ctx: &mut Engine, tiles: &Vec<Image>, center: Location) -> ChartPos {
    let mouse = ctx.get_mouse();
    let xf = Transform::new(ChartPos::from_location(center));
    let cursor_chart_pos = xf.to_chart(&mouse.pos);

    ctx.set_color(&FIREBRICK);
    ctx.set_layer(FLOOR_Z);
    ctx.draw_image(tiles.get(CURSOR_BOTTOM), &xf.to_screen(cursor_chart_pos));
    ctx.set_layer(BLOCK_Z);
    ctx.draw_image(tiles.get(CURSOR_TOP), &xf.to_screen(cursor_chart_pos));

    cursor_chart_pos
}
