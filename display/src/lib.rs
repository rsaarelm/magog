mod brush;
mod cache;
pub use crate::cache::font;
mod console;
mod init;
pub use crate::init::load_graphics;
mod render;
mod sprite;
mod view;

pub use crate::console::Console;
pub use crate::view::WorldView;

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
