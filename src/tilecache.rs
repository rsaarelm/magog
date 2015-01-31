use std::cell::RefCell;
use image;
use image::{SubImage, GenericImage};
use backend::{CanvasBuilder, Image};
use util::{color_key, V2, Rgb};

thread_local!(static TILE_CACHE: RefCell<Vec<Image>> = RefCell::new(vec![]));

fn batch(tiles: &mut Vec<Image>, ctx: &mut CanvasBuilder, data: &[u8],
       elt_dim: (i32, i32), offset: (i32, i32)) {
    let mut image = color_key(
        &image::load_from_memory(data).unwrap(),
        &Rgb::new(0x00u8, 0xFFu8, 0xFFu8));
    let (w, h) = image.dimensions();
    let (columns, rows) = (w / elt_dim.0 as u32, h / elt_dim.1 as u32);

    for y in 0..(rows) {
        for x in 0..(columns) {
            tiles.push(ctx.add_image(V2(offset.0, offset.1), &SubImage::new(
                &mut image,
                x * elt_dim.0 as u32, y * elt_dim.1 as u32,
                elt_dim.0 as u32, elt_dim.1 as u32)));
        }
    }
}

// XXX Copy-paste code to get alternating offsets for wall tiles.
fn batch2(tiles: &mut Vec<Image>, ctx: &mut CanvasBuilder, data: &[u8],
       elt_dim: (i32, i32), offset: (i32, i32)) {
    let mut image = color_key(
        &image::load_from_memory(data).unwrap(),
        &Rgb::new(0x00u8, 0xFFu8, 0xFFu8));
    let (w, h) = image.dimensions();
    let (columns, rows) = (w / elt_dim.0 as u32, h / elt_dim.1 as u32);

    for y in 0..(rows) {
        for x in 0..(columns) {
            let offset = V2(offset.0 + if x % 2 == 1 { 16 } else { 0 }, offset.1);
            tiles.push(ctx.add_image(offset, &SubImage::new(
                &mut image,
                x * elt_dim.0 as u32, y * elt_dim.1 as u32,
                elt_dim.0 as u32, elt_dim.1 as u32)));
        }
    }
}

/// Initialize global tile cache.
pub fn init(ctx: &mut CanvasBuilder) {
    TILE_CACHE.with(|c| {
        let mut tiles = c.borrow_mut();
        batch(&mut *tiles, ctx, include_bytes!("../assets/tile.png"), (32, 32), (-16, -16));
        batch2(&mut *tiles, ctx, include_bytes!("../assets/wall.png"), (16, 32), (-16, -16));
        batch(&mut *tiles, ctx, include_bytes!("../assets/icon.png"), (8, 8), (0, -8));
        batch(&mut *tiles, ctx, include_bytes!("../assets/logo.png"), (92, 25), (0, 0));
    });
}

pub fn get(idx: usize) -> Image {
    TILE_CACHE.with(|c| c.borrow()[idx].clone())
}

pub mod tile {
    pub static CUBE : usize = 0;
    pub static CURSOR_BOTTOM : usize = 1;
    pub static CURSOR_TOP : usize = 2;
    pub static BLOCK_NW : usize = 3;
    pub static BLOCK_N : usize = 4;
    pub static BLOCK_NE : usize = 5;
    pub static BLOCK_DARK : usize = 6;
    pub static CHASM : usize = 7;
    pub static SHALLOWS : usize = 8;
    pub static FLOOR_FRONT : usize = 9;
    pub static BLANK_FLOOR : usize = 10;
    pub static FLOOR : usize = 11;
    pub static GRASS : usize = 12;
    pub static WATER : usize = 13;
    pub static MAGMA : usize = 14;
    pub static DOWNSTAIRS : usize = 15;
    pub static ROCK : usize = 16;
    pub static BLANK_BLOCK : usize = 19;
    pub static TREE_TRUNK : usize = 48;
    pub static TREE_FOLIAGE : usize = 49;
    pub static TABLE : usize = 50;
    pub static AVATAR : usize = 51;
    pub static FOUNTAIN : usize = 53;
    pub static ALTAR : usize = 54;
    pub static BARREL : usize = 55;
    pub static STALAGMITE : usize = 56;
    pub static GRAVE : usize = 58;
    pub static SPLATTER : usize = 64;
    pub static STONE : usize = 69;
    pub static MENHIR : usize = 70;
    pub static TALLGRASS : usize = 80;
    pub static XYWALL : usize = 82;

    // Wall tiles
    pub static WALL : usize = 256;
    pub static WINDOW : usize = 274;
    pub static DOOR : usize = 262;
    pub static BARS : usize = 288;

    // TODO: Make own tiles for these
    pub static ROCKWALL : usize = 256;
    pub static BATTLEMENT : usize = 256;
}

pub mod icon {
    pub static HEART : usize = 384;
    pub static HALF_HEART : usize = 385;
    pub static NO_HEART : usize = 386;
    pub static SHARD : usize = 387;
    pub static HALF_SHARD : usize = 388;
    pub static NO_SHARD : usize = 389;
}

pub static LOGO: usize = 256 + 128 + 128;
