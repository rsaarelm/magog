/*!
Window-wrangling, polygon-pushing and input-grabbing.

*/
#![crate_name="backend"]
#![allow(unstable)]
#![feature(plugin)]

extern crate glutin;
#[plugin]
extern crate glium_macros;
extern crate glium;
extern crate util;
extern crate time;
extern crate image;

pub use canvas::{Canvas, Context};
pub use canvas::{Image};
pub use canvas_util::{CanvasUtil};
pub use key::Key;
pub use fonter::{Fonter, CanvasWriter};
pub use event::{Event};

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
