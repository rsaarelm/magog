use std::cell::RefCell;
use cgmath::vector::{Vector2};
use calx::stb::image;
use calx::tile::Tile;
use calx::engine::{Engine, Image};

local_data_key!(TILE_CACHE: RefCell<Vec<Image>>)

static TILE_DATA: &'static [u8] = include_bin!("../../assets/tile.png");

pub fn init(ctx: &mut Engine) {
    let tiles = image::Image::load_from_memory(TILE_DATA, 1).unwrap();
    let tiles = Tile::new_alpha_set(
        &Vector2::new(32, 32),
        &Vector2::new(tiles.width as int, tiles.height as int),
        tiles.pixels,
        &Vector2::new(-16, -16));
    let tiles = ctx.make_images(&tiles);

    TILE_CACHE.replace(Some(RefCell::new(tiles)));
}

pub fn get(idx: uint) -> Image {
    assert!(TILE_CACHE.get().is_some(), "Tile cache not initialized");
    TILE_CACHE.get().unwrap().borrow().get(idx).clone()
}
