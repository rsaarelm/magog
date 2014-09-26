use std::cell::RefCell;
use image;
use calx::{V2, Canvas, Image};

local_data_key!(TILE_CACHE: RefCell<Vec<Image>>)

fn batch(ctx: &mut Canvas, data: &[u8],
       elt_dim: (int, int), offset: (int, int)) -> Vec<Image> {
       /*
    let image = image::load_from_memory(data, image::PNG).unwrap();
    let (elt_w, elt_h) = image.dimensions();
    let (offset_w, offset_h) = offset;

    let set = Tile::new_alpha_set(
            &Vector2::new(elt_w, elt_h),
            &Vector2::new(image.width as int, image.height as int),
            image.pixels,
            &Vector2::new(offset_w, offset_h));
    let ret = set.len();
    tiles.push_all(set.as_slice());
    ret
    */
    // TODO
    vec![]
}

/// Initialize global tile cache.
pub fn init(ctx: &mut Canvas) {
    let mut tiles = vec![];
    tiles.extend(batch(ctx, include_bin!("../../assets/tile.png"), (32, 32), (-16, -16)).iter());
    tiles.extend(batch(ctx, include_bin!("../../assets/icon.png"), (8, 8), (0, -8)).iter());
    tiles.extend(batch(ctx, include_bin!("../../assets/logo.png"), (92, 25), (0, 0)).iter());
    TILE_CACHE.replace(Some(RefCell::new(tiles)));
}

pub fn get(idx: uint) -> Image {
    assert!(TILE_CACHE.get().is_some(), "Tile cache not initialized");
    (*TILE_CACHE.get().unwrap().borrow())[idx].clone()
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
    pub static STONE : uint = 69;
    pub static MENHIR : uint = 70;
    pub static TALLGRASS : uint = 80;
}

pub mod icon {
    static ICON_OFFSET: uint = 256;

    pub static HEART : uint = ICON_OFFSET + 0;
    pub static HALF_HEART : uint = ICON_OFFSET + 1;
    pub static NO_HEART : uint = ICON_OFFSET + 2;
    pub static SHARD : uint = ICON_OFFSET + 3;
    pub static HALF_SHARD : uint = ICON_OFFSET + 4;
    pub static NO_SHARD : uint = ICON_OFFSET + 5;
}

pub static LOGO: uint = 256 + 128;
