mod backend;
pub use crate::backend::{Backend, Core, KeyEvent};

mod brush;
mod cache;
pub use crate::cache::font;
mod console;

mod draw_util;
pub use crate::draw_util::DrawUtil;

pub mod init;
mod render;
mod sprite;
mod view;

pub use crate::console::Console;
pub use crate::view::WorldView;

type SubImageSpec = vitral::SubImageSpec<String>;
type AtlasCache = vitral::AtlasCache<String>;

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
