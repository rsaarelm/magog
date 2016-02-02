/*!
Miscellaneous utilities grab-bag.

 */

#![crate_name="calx"]

#![feature(deprecated)]
#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate serde;
extern crate time;
extern crate rand;
extern crate num;
extern crate vec_map;
extern crate nalgebra;
extern crate image;
extern crate bincode;
extern crate calx_alg;

#[macro_use] extern crate glium;

use std::path::{Path, PathBuf};

pub use rgb::{Rgba, SRgba, color, scolor, NAMED_COLORS};
pub use geom::{V2, V3, Rect, RectIter, IterTiles};

#[deprecated] mod geom;
#[deprecated] mod rgb;
pub mod backend;
pub mod debug;
pub mod timing;

/// Rectangle anchoring points.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Anchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Top,
    Left,
    Right,
    Bottom,
    Center
}

/// Return the application data directory path for the current platform.
pub fn app_data_path(app_name: &str) -> PathBuf {
    use std::env;
    // On Windows, a portable application is just an .exe the user downloads
    // and drops somewhere. The convention here is for a portable application
    // to add its files to wherever its exe file is. An installed application
    // uses an actual installer program and deploys its files to user data
    // directories.
    let is_portable_application = true;

    // TODO: Handle not having the expected env variables.
    if cfg!(windows) {
        if is_portable_application {
            match env::current_exe() {
                Ok(mut p) => { p.pop(); p }
                // If couldn't get self exe path, just use the local relative path and
                // hope for the best.
                _ => Path::new(".").to_path_buf()
            }
        } else {
            Path::new(
                &format!("{}\\{}", env::var("APPDATA").unwrap(), app_name))
            .to_path_buf()
        }
    } else if cfg!(macos) {
        Path::new(
            &format!("{}/Library/Application Support/{}",
                    env::var("HOME").unwrap(), app_name))
        .to_path_buf()
    } else {
        Path::new(
            &format!("{}/.config/{}", env::var("HOME").unwrap(), app_name))
        .to_path_buf()
    }
}

#[macro_export]
macro_rules! count_exprs {
    () => { 0 };
    ($e:expr) => { 1 };
    ($e:expr, $($es:expr),+) => { 1 + count_exprs!($($es),*) };
}
