/*!
Window-wrangling, polygon-pushing and input-grabbing.

*/
#![crate_name="calx_backend"]
#![feature(collections, old_io, std_misc)]
#![feature(plugin)]
#![plugin(glium_macros)]

extern crate glutin;

#[macro_use]
extern crate glium;
extern crate "calx_util" as util;
extern crate time;
extern crate image;

pub use canvas::{CanvasBuilder, Canvas};
pub use canvas::{Image};
pub use canvas_util::{CanvasUtil};
pub use key::Key;
pub use fonter::{Fonter, Align};
pub use event::{Event, MouseButton};

mod canvas;
mod canvas_util;
mod event;
mod fonter;
mod key;
mod renderer;

#[cfg(target_os = "macos")]
mod scancode_macos;
#[cfg(target_os = "linux")]
mod scancode_linux;
#[cfg(target_os = "windows")]
mod scancode_windows;

mod scancode {
#[cfg(target_os = "macos")]
    pub use scancode_macos::MAP;
#[cfg(target_os = "linux")]
    pub use scancode_linux::MAP;
#[cfg(target_os = "windows")]
    pub use scancode_windows::MAP;
}

/// UI Widget static identifier, unique for a specific site in source code.
#[derive(Copy, Debug, PartialEq)]
pub struct WidgetId {
    filename: &'static str,
    line: usize,
    column: usize,
}

impl WidgetId {
    pub fn new(filename: &'static str, line: usize, column: usize) -> WidgetId {
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
        // XXX: Assuming the crate user renames 'calx_backend' to 'backend'
        // when declaring the extern crate, since the app that does use this
        // does that. This is extremely wrong and bad to assume. Don't know
        // how to do this right.
        backend::WidgetId::new(concat!(module_path!(), "/", file!()), line!(), column!())
    }
}
