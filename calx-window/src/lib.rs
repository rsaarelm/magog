extern crate time;
extern crate image;
#[macro_use]
extern crate glium;
extern crate calx_layout;
extern crate calx_color;
extern crate calx_alg;

pub use key::Key;
pub use event::{Event, MouseButton};
pub use window::{WindowBuilder, Window};

mod event;
mod event_translator;
mod key;
mod window;

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

/// Things that draw themselves on the Window
pub trait Displayable {
    fn display<S>(&mut self, display: &glium::Display, target: &mut S)
        where S: glium::Surface;
}
