/*!
Window-wrangling, polygon-pushing and input-grabbing.

*/

pub use backend::canvas::{CanvasBuilder, Canvas};
pub use backend::canvas::{Image};
pub use backend::canvas_util::{CanvasUtil};
pub use backend::key::Key;
pub use backend::fonter::{Fonter, Align};
pub use backend::event::{Event, MouseButton};
pub use backend::sprite_cache::{SpriteCache, SpriteKey};

mod canvas;
mod canvas_util;
mod event;
mod fonter;
mod key;
mod renderer;
mod sprite_cache;

#[cfg(target_os = "macos")]
mod scancode_macos;
#[cfg(target_os = "linux")]
mod scancode_linux;
#[cfg(target_os = "windows")]
mod scancode_windows;

mod scancode {
#[cfg(target_os = "macos")]
    pub use backend::scancode_macos::MAP;
#[cfg(target_os = "linux")]
    pub use backend::scancode_linux::MAP;
#[cfg(target_os = "windows")]
    pub use backend::scancode_windows::MAP;
}

#[derive(Copy, Clone, Debug, PartialEq)]
/// How to scale up the graphics to a higher resolution
pub enum CanvasMagnify {
    /// Nearest-neighbor, fill the window, not pixel-perfect
    Nearest,
    /// Pixel-perfect nearest neighbor, only magnify to the largest full
    /// multiple of pixel size that fits on the window
    PixelPerfect,
    /// Use smooth filtering, may look blurry
    Smooth,
}

/// UI Widget static identifier, unique for a specific site in source code.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WidgetId {
    filename: &'static str,
    line: u32,
    column: u32,
}

impl WidgetId {
    pub fn new(filename: &'static str, line: u32, column: u32) -> WidgetId {
        WidgetId {
            filename: filename,
            line: line,
            column: column,
        }
    }

    pub fn dummy() -> WidgetId {
        WidgetId {
            filename: "n/a",
            line: 666666,
            column: 666666,
        }
    }
}

#[macro_export]
/// Generate a static identifier for the current source code position. Used
/// with imgui API.
macro_rules! widget_id {
    () => {
        ::calx::backend::WidgetId::new(concat!(module_path!(), "/", file!()), line!(), column!())
    }
}
