use crate::brush::Brush;
use crate::init;
use crate::Icon;
use lazy_static::lazy_static;
use std::sync::Arc;
use vec_map::VecMap;
use vitral::{self, FontData, PngBytes};
use world;

lazy_static! {
    static ref TERRAIN_BRUSHES: VecMap<Arc<Brush>> = init::terrain_brushes();
    static ref ENTITY_BRUSHES: VecMap<Arc<Brush>> = init::entity_brushes();
    static ref MISC_BRUSHES: VecMap<Arc<Brush>> = init::misc_brushes();
    static ref FONT: Arc<FontData> = Arc::new(vitral::add_tilesheet_font(
        "font",
        PngBytes(include_bytes!("../assets/font.png")),
        (32u8..128).map(|c| c as char)
    ));
}

pub fn terrain(t: world::Terrain) -> Arc<Brush> {
    Arc::clone(
        TERRAIN_BRUSHES
            .get(t as usize)
            .unwrap_or_else(|| panic!("No brush for terrain {:?}", t)),
    )
}

pub fn entity(e: world::Icon) -> Arc<Brush> {
    Arc::clone(
        ENTITY_BRUSHES
            .get(e as usize)
            .unwrap_or_else(|| panic!("No brush for entity {:?}", e)),
    )
}

pub fn misc(e: Icon) -> Arc<Brush> {
    Arc::clone(
        MISC_BRUSHES
            .get(e as usize)
            .unwrap_or_else(|| panic!("No brush for icon {:?}", e)),
    )
}

pub fn font() -> Arc<FontData> { Arc::clone(&*FONT) }
