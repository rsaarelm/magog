mod brush;
mod cache;
mod canvas_ext;
pub use cache::font;
mod console;
mod init;
pub use init::load_graphics;
mod render;
mod sprite;
mod view;

pub use canvas_ext::CanvasExt;
pub use console::Console;
pub use view::{ScreenVector, WorldView};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Icon {
    SolidBlob,
    CursorTop,
    CursorBottom,
    Portal,
    HealthPip,
    DarkHealthPip,
    Gib,
    Smoke,
    Explosion,
    Firespell,
}
