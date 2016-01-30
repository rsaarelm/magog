/*!
Miscellaneous utilities grab-bag.

 */

#![crate_name="calx"]

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
pub use fs::{PathExt};
pub use geom::{V2, V3, Rect, RectIter, IterTiles};
pub use img::{color_key, ImageStore};
pub use atlas::{AtlasBuilder, Atlas, AtlasItem};
pub use hex::{HexGeom, Dir6, HexFov, Dir12};
pub use index_cache::{IndexCache, CacheKey};
pub use kernel::{Kernel, KernelTerrain};
pub use projection::{Projection};

mod atlas;
mod brush;
mod fs;
mod geom;
mod hex;
mod img;
mod index_cache;
mod kernel;
mod projection;
mod rgb;

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

/// Exponential moving average duration.
pub struct AverageDuration {
    weight: f64,
    last_time: f64,
    pub value: f64,
}

impl AverageDuration {
    /// Init is the initial value for the duration, somewhere in the scale you
    /// expect the actual values to be. Weight is between 0 and 1 and
    /// indicates how fast the older values should decay. Weight 1.0 causes
    /// old values to decay immediately.
    pub fn new(init: f64, weight: f64) -> AverageDuration {
        assert!(weight > 0.0 && weight <= 1.0);
        AverageDuration {
            weight: weight,
            last_time: time::precise_time_s(),
            value: init,
        }
    }

    pub fn tick(&mut self) {
        let t = time::precise_time_s();
        self.value = self.weight * (t - self.last_time) + (1.0 - self.weight) * self.value;
        self.last_time = t;
    }
}

#[macro_export]
macro_rules! count_exprs {
    () => { 0 };
    ($e:expr) => { 1 };
    ($e:expr, $($es:expr),+) => { 1 + count_exprs!($($es),*) };
}
