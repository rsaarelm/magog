extern crate num;
extern crate vec_map;
extern crate image;
extern crate calx_layout;
extern crate calx_color;

pub use img::{color_key, ImageStore, subimage, tilesheet_bounds};
pub use atlas::{AtlasBuilder, Atlas, AtlasItem};
pub use index_cache::{IndexCache, CacheKey};

mod atlas;
mod brush;
mod img;
mod index_cache;
