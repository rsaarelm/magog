/*!
Miscellaneous utilities grab-bag.

 */

#![crate_name="calx"]
#![feature(core, collections, path_ext, std_misc)]
#![feature(plugin, custom_attribute, unboxed_closures, slice_patterns)]
#![feature(custom_derive)]
#![plugin(rand_macros)]

#[no_link] extern crate rand_macros;
extern crate collections;
extern crate rustc_serialize;
extern crate time;
extern crate rand;
extern crate num;
extern crate nalgebra;
extern crate image;
extern crate glutin;
#[macro_use]
extern crate glium;

use num::{Float, Zero, One};
use std::path::{Path, PathBuf};
use std::ops::{Add, Sub, Mul};
use std::cmp::{Ordering};
use num::traits::{Num};

pub use rgb::{ToColor, FromColor, Rgba, color, parse_color};
pub use geom::{V2, V3, Rect, RectIter};
pub use img::{color_key};
pub use atlas::{AtlasBuilder, Atlas, AtlasItem};
pub use dijkstra::{DijkstraNode, Dijkstra};
pub use hex::{HexGeom, Dir6, HexFov};
pub use projection::{Projection};
pub use rng::{EncodeRng, RngExt};

mod atlas;
mod dijkstra;
mod geom;
mod hex;
mod img;
mod projection;
mod rgb;
mod rng;

pub mod backend;
pub mod text;
pub mod timing;
pub mod vorud;

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
pub fn lerp<T, U>(t: T, a: U, b: U) -> U where
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

/// A non-transitive pseudo-ordering relation between isometric axis-aligned boxes.
///
/// XXX: The relation is NOT TRANSITIVE, the more disparate box sizes
/// you feed it the worse it's likely to perform.
///
/// The box format is (bottom_pos, top_pos) pairs. The camera looks towards (-1, -1, -1).
pub fn isometric_pseudo_ord<T: PartialOrd+Num+Copy>(
    &(ref first_bot, ref first_top): &(V3<T>, V3<T>),
    &(ref second_bot, ref second_top): &(V3<T>, V3<T>)) -> Ordering {
    // Return Less if first box is drawn before second box
    // Return Greater if first box is drawn after second box
    // If the boxes overlap, compare their top Z positions.

    // Look for the unambiguous order with the largest distance between
    // box faces.
    let mut less_weight = None;
    let mut greater_weight = None;

    less_weight = comp(second_bot.0, first_top.0, less_weight);
    less_weight = comp(second_bot.1, first_top.1, less_weight);
    less_weight = comp(second_bot.2, first_top.2, less_weight);

    greater_weight = comp(first_bot.0, second_top.0, greater_weight);
    greater_weight = comp(first_bot.1, second_top.1, greater_weight);
    greater_weight = comp(first_bot.2, second_top.2, greater_weight);

    return match (less_weight, greater_weight) {
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (Some(ls), Some(gt)) => if ls > gt { Ordering::Less } else { Ordering::Greater },
        _ => {
            let camera: V3<T> = V3(One::one(), One::one(), One::one());
            (first_top).dot(camera).partial_cmp(&second_top.dot(camera))
            .unwrap_or(Ordering::Equal)
        }
    };

    // Update a result weight counter with a comparison along an axis.
    fn comp<T: PartialOrd+Num+Copy>(bottom_point: T, top_point: T, weight_counter: Option<T>) -> Option<T> {
        let diff = bottom_point - top_point ;
        if diff >= Zero::zero() {
            Some(weight_counter.map_or(diff, |old| if diff > old { diff } else { old }))
        } else {
            weight_counter
        }
    }

}

#[cfg(test)]
mod test {
    use super::geom::{V3};

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

    fn gen_boxes(positions: Vec<V3<i32>>) -> Vec<(V3<i32>, V3<i32>)> {
        positions.into_iter().map(|p| (p, p + V3(2, 2, 2))).collect()
    }

    #[test]
    fn test_iso_sort() {
        use super::{isometric_pseudo_ord};

        // XXX: Just a sanity check, actually debugging the problems
        // with the slightly iffy sorter would take a lot more heavy
        // testing.
        let mut boxes = gen_boxes(vec![V3(0, 0, 2), V3(-2, 0, 0), V3(0, 0, 0)]);
        boxes.sort_by(isometric_pseudo_ord);

        assert_eq!(V3(-2, 0, 0), boxes[0].0);
        assert_eq!(V3(0, 0, 2), boxes[2].0);
    }
}
