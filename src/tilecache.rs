use std::cell::RefCell;
use image;
use image::{SubImage, GenericImage};
use backend::{Canvas, Image};
use util::{color_key, V2, Rgb};

thread_local!(static TILE_CACHE: RefCell<Vec<Image>> = RefCell::new(vec![]));

fn batch(tiles: &mut Vec<Image>, ctx: &mut Canvas, data: &[u8],
       elt_dim: (int, int), offset: (int, int)) {
    let mut image = color_key(
        &image::load_from_memory(data, image::PNG).unwrap(),
        &Rgb::new(0x00u8, 0xFFu8, 0xFFu8));
    let (w, h) = image.dimensions();
    let (columns, rows) = (w / elt_dim.0 as u32, h / elt_dim.1 as u32);

    for y in range(0, rows) {
        for x in range(0, columns) {
            tiles.push(ctx.add_image(V2(offset.0, offset.1), SubImage::new(
                &mut image,
                x * elt_dim.0 as u32, y * elt_dim.1 as u32,
                elt_dim.0 as u32, elt_dim.1 as u32)));
        }
    }
}

/// Initialize global tile cache.
pub fn init(ctx: &mut Canvas) {
    TILE_CACHE.with(|c| {
        let mut tiles = c.borrow_mut();
        batch(tiles.deref_mut(), ctx, include_bytes!("../assets/tile.png"), (32, 32), (-16, -16));
        batch(tiles.deref_mut(), ctx, include_bytes!("../assets/icon.png"), (8, 8), (0, -8));
        batch(tiles.deref_mut(), ctx, include_bytes!("../assets/logo.png"), (92, 25), (0, 0));
    });
}

pub fn get(idx: uint) -> Image {
    TILE_CACHE.with(|c| c.borrow()[idx].clone())
}

pub mod tile {
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
    pub static SPLATTER : uint = 64;
    pub static STONE : uint = 69;
    pub static MENHIR : uint = 70;
    pub static TALLGRASS : uint = 80;
    pub static XYWALL : uint = 82;
}

pub mod icon {
    pub static HEART : uint = 256;
    pub static HALF_HEART : uint = 257;
    pub static NO_HEART : uint = 258;
    pub static SHARD : uint = 259;
    pub static HALF_SHARD : uint = 260;
    pub static NO_SHARD : uint = 261;
}

pub static LOGO: uint = 256 + 128;
