extern crate time;
extern crate euclid;
#[macro_use]
extern crate glium;
extern crate image;
extern crate vec_map;
extern crate vitral;
extern crate vitral_atlas;
extern crate world;
extern crate calx_grid;
extern crate calx_color;
extern crate calx_alg;

mod atlas_cache;
mod backend;
mod brush;
mod cache;
mod canvas;
mod canvas_zoom;
mod console;
pub mod init;
mod render;
mod sprite;
mod tilesheet;
mod view;

pub use backend::Backend;
pub use canvas_zoom::CanvasZoom;
pub use console::Console;
pub use view::WorldView;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Icon {
    SolidBlob,
    CursorTop,
    CursorBottom,
    Portal,
}
