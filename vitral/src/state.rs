//! Stuff that uses the global state.
use crate::{atlas_cache::AtlasCache, Flick, FontData, ImageData, ImageKey};
use lazy_static::lazy_static;
use std::sync::Mutex;

#[derive(Default)]
pub struct EngineState {
    pub atlas_cache: AtlasCache<String>,
    pub average_frame_duration: Flick,
}

lazy_static! {
    /// Global game engine state.
    pub static ref ENGINE_STATE: Mutex<EngineState> = { Mutex::new(EngineState::default()) };
}

/// Return the average frame duration for recent frames.
///
/// Panics if called when an app isn't running via `run_app`.
pub fn get_frame_duration() -> Flick {
    ENGINE_STATE.lock().unwrap().average_frame_duration
}

/// Add a named image into the engine image atlas.
pub fn add_sheet(
    id: impl Into<String>,
    sheet: impl Into<image::RgbaImage>,
) -> ImageKey {
    ENGINE_STATE
        .lock()
        .unwrap()
        .atlas_cache
        .add_sheet(id, sheet)
}

/// Add a tilesheet image that gets automatically split to subimages based on image structure.
///
/// Tiles are bounding boxes of non-background pixel groups surrounded by only background pixels or
/// image edges. Background color is the color of the bottom right corner pixel of the image. The
/// bounding boxes are returned lexically sorted by the coordinates of their bottom right corners,
/// first along the y-axis then along the x-axis. This produces a natural left-to-right,
/// bottom-to-top ordering for a cleanly laid out tile sheet.
///
/// Note that the background color is a solid color, not transparent pixels. The inner tiles may
/// have transparent parts, so a solid color is needed to separate them.
pub fn add_tilesheet(
    id: impl Into<String>,
    sheet: impl Into<image::RgbaImage>,
    _span: impl IntoIterator<Item = char>,
) -> Vec<ImageKey> {
    ENGINE_STATE
        .lock()
        .unwrap()
        .atlas_cache
        .add_tilesheet(id, sheet)
}

/// Add a bitmap font read from a tilesheet image.
pub fn add_tilesheet_font(
    id: impl Into<String>,
    sheet: impl Into<image::RgbaImage>,
    span: impl IntoIterator<Item = char>,
) -> FontData {
    ENGINE_STATE
        .lock()
        .unwrap()
        .atlas_cache
        .add_tilesheet_font(id, sheet, span)
}

/// Get a drawable (sub)image from the cache corresponding to the given `ImageKey`.
///
/// If the `ImageKey` specifies a sheet not found in the cache or invalid dimensions, will return
/// `None`.
pub fn get_image(key: &ImageKey) -> Option<ImageData> {
    ENGINE_STATE.lock().unwrap().atlas_cache.get(key)
}
