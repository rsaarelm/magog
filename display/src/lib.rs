extern crate time;
extern crate euclid;
#[macro_use]
extern crate glium;
extern crate image;
extern crate vec_map;
extern crate vitral;
extern crate world;
extern crate calx_ecs;
extern crate calx;

mod atlas_cache;

mod backend;
pub use backend::{Backend, Core, KeyEvent};

mod brush;
mod cache;
pub use cache::font;
mod console;

mod draw_util;
pub use draw_util::DrawUtil;

pub mod init;
mod render;
mod sprite;
mod tilesheet;
mod view;

pub use console::Console;
pub use view::WorldView;

pub type ImageData = vitral::ImageData<vitral::glium_backend::TextureHandle>;
pub type FontData = vitral::FontData<vitral::glium_backend::TextureHandle>;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Icon {
    SolidBlob,
    CursorTop,
    CursorBottom,
    Portal,
    HealthPip,
    DarkHealthPip,
    BlockedOffSectorCell,
}
