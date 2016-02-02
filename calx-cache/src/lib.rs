extern crate image;
extern crate calx_layout;
extern crate vec_map;

pub use img::{color_key, ImageStore};
pub use atlas::{AtlasBuilder, Atlas, AtlasItem};
pub use index_cache::{IndexCache, CacheKey};

mod atlas;
mod brush;
mod img;
mod index_cache;
