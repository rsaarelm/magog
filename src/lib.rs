#![crate_name="calx"]
#![feature(phase)]
#![feature(globs)]

extern crate time;
extern crate collections;
extern crate "rustc-serialize" as rustc_serialize;

extern crate glutin;
extern crate gfx;
#[phase(plugin)]
extern crate gfx_macros;
extern crate device;
extern crate image;

pub use canvas::{Canvas, Context};
pub use canvas::{Image};
pub use canvas_util::{CanvasUtil};
pub use rgb::{Rgb, Rgba};
pub use geom::{Rect, V2, RectIter};
pub use util::{color_key};
pub use util::{Primitive};
pub use key::Key;
pub use fonter::{Fonter, CanvasWriter};
pub use event::{Event};

mod atlas;
mod canvas;
mod canvas_util;
mod event;
mod fonter;
mod geom;
mod key;
mod renderer;
mod rgb;

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

mod util;
pub mod color;
pub mod dijkstra;
pub mod text;
pub mod timing;
pub mod vorud;

pub trait Color {
    fn to_rgba(&self) -> [f32, ..4];
}
