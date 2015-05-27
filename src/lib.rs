/*!
Miscellaneous utilities grab-bag.

 */

#![crate_name="calx"]

extern crate rustc_serialize;
extern crate time;
extern crate rand;
extern crate num;
extern crate vec_map;
extern crate nalgebra;
extern crate image;
extern crate glutin;

#[macro_use] extern crate glium;

#[macro_use] extern crate calx_ecs;

use num::{Float};
use std::path::{Path, PathBuf};
use std::ops::{Add, Sub, Mul};

pub use rgb::{Rgba, SRgba, color, scolor, NAMED_COLORS};
pub use fs::{PathExt};
pub use geom::{V2, V3, Rect, RectIter, IterTiles};
pub use img::{color_key};
pub use atlas::{AtlasBuilder, Atlas, AtlasItem};
pub use search::{LatticeNode, Dijkstra, astar_path_with};
pub use hex::{HexGeom, Dir6, HexFov};
pub use kernel::{Kernel, KernelTerrain};
pub use projection::{Projection};
pub use rng::{EncodeRng, RngExt};

mod atlas;
mod fs;
mod geom;
mod hex;
mod img;
mod kernel;
mod projection;
mod rgb;
mod rng;
mod search;

pub mod backend;
pub mod ease;
pub mod text;
pub mod timing;
pub mod vorud;

#[cfg(test)]
mod test_ecs;

/// Clamp a value to range.
pub fn clamp<C: PartialOrd+Copy>(mn: C, mx: C, x: C) -> C {
    if x < mn { mn }
    else if x > mx { mx }
    else { x }
}

/// Deterministic noise.
pub fn noise(n: i32) -> f32 {
    use std::num::Wrapping;

    let n = Wrapping(n);
    let n = (n << 13) ^ n;
    let m = (n * (n * n * Wrapping(15731) + Wrapping(789221)) + Wrapping(1376312589))
        & Wrapping(0x7fffffff);
    let Wrapping(m) = m;
    1.0 - m as f32 / 1073741824.0
}

/// Convert probability to a log odds deciban value.
///
/// Log odds correspond to the Bayesian probability for a hypothesis that
/// has decibans * 1/10 log_2(10) bits of evidence in favor of it. They're
/// a bit like rolling a d20 but better.
pub fn to_log_odds(p: f32) -> f32 {
    10.0 * (p / (1.0 - p)).log(10.0)
}

/// Convert a log odds deciban value to the corresponding probability.
pub fn from_log_odds(db: f32) -> f32 {
    (1.0 - 1.0 / (1.0 + 10.0.powf(db / 10.0)))
}

/// Interpolate linearly between two values.
pub fn lerp<T, U>(a: U, b: U, t: T) -> U where
        U: Add<U, Output=U> + Sub<U, Output=U> + Mul<T, Output=U> + Copy {
    a + (b - a) * t
}

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


#[cfg(test)]
mod test {
    #[test]
    fn test_noise() {
        use super::noise;

        for i in 0i32..100 {
            assert!(noise(i) >= -1.0 && noise(i) <= 1.0);
        }
    }

    #[test]
    fn test_log_odds() {
        use super::{to_log_odds, from_log_odds};
        assert_eq!(from_log_odds(0.0), 0.5);
        assert_eq!(to_log_odds(0.5), 0.0);

        assert_eq!((from_log_odds(-5.0) * 100.0) as i32, 24);
        assert_eq!(to_log_odds(0.909091) as i32, 10);
    }
}
